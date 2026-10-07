//! # Subscriptions
//!
//! Recurring pull payments for any SEP-41 token on Soroban.
//!
//! A **merchant** publishes a *plan* (token, amount, period). A **subscriber**
//! approves this contract as a token spender (`token.approve`) and subscribes.
//! The first period is charged immediately; later periods can be charged by
//! *anyone* (a keeper bot) once they are due. Funds move directly
//! `subscriber -> merchant` with `transfer_from`; the contract never holds
//! user funds. The subscriber can cancel at any time, and the token allowance
//! is the hard upper bound on what can ever be pulled.
#![no_std]

use soroban_sdk::{
    contract, contracterror, contractevent, contractimpl, contracttype, token, Address, Env,
};

#[cfg(test)]
mod test;

/// Shortest allowed billing period, in seconds.
pub const MIN_PERIOD: u64 = 60;

const DAY_IN_LEDGERS: u32 = 17_280;
const BUMP_THRESHOLD: u32 = 29 * DAY_IN_LEDGERS;
const BUMP_AMOUNT: u32 = 30 * DAY_IN_LEDGERS;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    InvalidAmount = 1,
    InvalidPeriod = 2,
    PlanNotFound = 3,
    PlanInactive = 4,
    SubscriptionNotFound = 5,
    NotDue = 6,
    AlreadyCancelled = 7,
    Overflow = 8,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Plan {
    pub merchant: Address,
    pub token: Address,
    /// Amount charged each period, in the token's smallest unit.
    pub amount: i128,
    /// Seconds between charges.
    pub period: u64,
    pub active: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Subscription {
    pub plan_id: u32,
    pub subscriber: Address,
    /// Earliest ledger timestamp at which the next charge is allowed.
    pub next_charge: u64,
    /// Number of successful charges so far (including the first).
    pub charges: u32,
    pub cancelled: bool,
}

#[contractevent]
pub struct PlanCreated {
    #[topic]
    pub plan_id: u32,
    pub merchant: Address,
    pub amount: i128,
    pub period: u64,
}

#[contractevent]
pub struct PlanDeactivated {
    #[topic]
    pub plan_id: u32,
}

#[contractevent]
pub struct Subscribed {
    #[topic]
    pub sub_id: u32,
    pub subscriber: Address,
    pub plan_id: u32,
}

#[contractevent]
pub struct Charged {
    #[topic]
    pub sub_id: u32,
    pub charges: u32,
}

#[contractevent]
pub struct Cancelled {
    #[topic]
    pub sub_id: u32,
}

#[contracttype]
#[derive(Clone)]
enum DataKey {
    PlanCount,
    SubCount,
    Plan(u32),
    Sub(u32),
}

#[contract]
pub struct Subscriptions;

#[contractimpl]
impl Subscriptions {
    /// Publish a billing plan. Only the merchant authorizes this.
    pub fn create_plan(
        env: Env,
        merchant: Address,
        token: Address,
        amount: i128,
        period: u64,
    ) -> Result<u32, Error> {
        merchant.require_auth();
        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }
        if period < MIN_PERIOD {
            return Err(Error::InvalidPeriod);
        }
        bump_instance(&env);
        let id = next_id(&env, &DataKey::PlanCount)?;
        let plan = Plan {
            merchant: merchant.clone(),
            token,
            amount,
            period,
            active: true,
        };
        save_plan(&env, id, &plan);
        PlanCreated {
            plan_id: id,
            merchant,
            amount,
            period,
        }
        .publish(&env);
        Ok(id)
    }

    /// Stop accepting new subscribers and block further charges on a plan.
    pub fn deactivate_plan(env: Env, plan_id: u32) -> Result<(), Error> {
        let mut plan = load_plan(&env, plan_id)?;
        plan.merchant.require_auth();
        bump_instance(&env);
        plan.active = false;
        save_plan(&env, plan_id, &plan);
        PlanDeactivated { plan_id }.publish(&env);
        Ok(())
    }

    /// Subscribe and pay the first period immediately.
    ///
    /// The subscriber must have approved this contract on the plan's token for
    /// at least `plan.amount` beforehand, otherwise the token call fails and
    /// the whole transaction reverts.
    pub fn subscribe(env: Env, subscriber: Address, plan_id: u32) -> Result<u32, Error> {
        subscriber.require_auth();
        let plan = load_plan(&env, plan_id)?;
        if !plan.active {
            return Err(Error::PlanInactive);
        }
        bump_instance(&env);
        let now = env.ledger().timestamp();
        let next_charge = now.checked_add(plan.period).ok_or(Error::Overflow)?;
        let sub_id = next_id(&env, &DataKey::SubCount)?;
        let sub = Subscription {
            plan_id,
            subscriber: subscriber.clone(),
            next_charge,
            charges: 1,
            cancelled: false,
        };
        save_sub(&env, sub_id, &sub);
        // State is written first; any failure below reverts the transaction.
        pull(&env, &plan, &subscriber);
        Subscribed {
            sub_id,
            subscriber,
            plan_id,
        }
        .publish(&env);
        Ok(sub_id)
    }

    /// Charge one due period. Callable by anyone (keeper-friendly).
    ///
    /// Missed periods are **not** back-charged: the next due time becomes
    /// `now + period`, so at most one period is pulled per call.
    pub fn charge(env: Env, sub_id: u32) -> Result<(), Error> {
        let mut sub = load_sub(&env, sub_id)?;
        if sub.cancelled {
            return Err(Error::AlreadyCancelled);
        }
        let plan = load_plan(&env, sub.plan_id)?;
        if !plan.active {
            return Err(Error::PlanInactive);
        }
        let now = env.ledger().timestamp();
        if now < sub.next_charge {
            return Err(Error::NotDue);
        }
        bump_instance(&env);
        sub.next_charge = now.checked_add(plan.period).ok_or(Error::Overflow)?;
        sub.charges = sub.charges.checked_add(1).ok_or(Error::Overflow)?;
        save_sub(&env, sub_id, &sub);
        pull(&env, &plan, &sub.subscriber);
        Charged {
            sub_id,
            charges: sub.charges,
        }
        .publish(&env);
        Ok(())
    }

    /// Cancel a subscription. Only the subscriber authorizes this.
    pub fn cancel(env: Env, sub_id: u32) -> Result<(), Error> {
        let mut sub = load_sub(&env, sub_id)?;
        sub.subscriber.require_auth();
        if sub.cancelled {
            return Err(Error::AlreadyCancelled);
        }
        bump_instance(&env);
        sub.cancelled = true;
        save_sub(&env, sub_id, &sub);
        Cancelled { sub_id }.publish(&env);
        Ok(())
    }

    pub fn get_plan(env: Env, plan_id: u32) -> Result<Plan, Error> {
        load_plan(&env, plan_id)
    }

    pub fn get_subscription(env: Env, sub_id: u32) -> Result<Subscription, Error> {
        load_sub(&env, sub_id)
    }

    /// `true` if `charge` would currently succeed on timing/state grounds.
    pub fn is_due(env: Env, sub_id: u32) -> Result<bool, Error> {
        let sub = load_sub(&env, sub_id)?;
        let plan = load_plan(&env, sub.plan_id)?;
        Ok(!sub.cancelled && plan.active && env.ledger().timestamp() >= sub.next_charge)
    }

    pub fn plan_count(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::PlanCount)
            .unwrap_or(0)
    }

    pub fn subscription_count(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::SubCount)
            .unwrap_or(0)
    }
}

fn pull(env: &Env, plan: &Plan, subscriber: &Address) {
    token::Client::new(env, &plan.token).transfer_from(
        &env.current_contract_address(),
        subscriber,
        &plan.merchant,
        &plan.amount,
    );
}

fn bump_instance(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(BUMP_THRESHOLD, BUMP_AMOUNT);
}

fn next_id(env: &Env, key: &DataKey) -> Result<u32, Error> {
    let current: u32 = env.storage().instance().get(key).unwrap_or(0);
    let next = current.checked_add(1).ok_or(Error::Overflow)?;
    env.storage().instance().set(key, &next);
    Ok(next)
}

fn save_plan(env: &Env, id: u32, plan: &Plan) {
    let key = DataKey::Plan(id);
    env.storage().persistent().set(&key, plan);
    env.storage()
        .persistent()
        .extend_ttl(&key, BUMP_THRESHOLD, BUMP_AMOUNT);
}

fn save_sub(env: &Env, id: u32, sub: &Subscription) {
    let key = DataKey::Sub(id);
    env.storage().persistent().set(&key, sub);
    env.storage()
        .persistent()
        .extend_ttl(&key, BUMP_THRESHOLD, BUMP_AMOUNT);
}

fn load_plan(env: &Env, id: u32) -> Result<Plan, Error> {
    env.storage()
        .persistent()
        .get(&DataKey::Plan(id))
        .ok_or(Error::PlanNotFound)
}

fn load_sub(env: &Env, id: u32) -> Result<Subscription, Error> {
    env.storage()
        .persistent()
        .get(&DataKey::Sub(id))
        .ok_or(Error::SubscriptionNotFound)
}
