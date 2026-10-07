#![cfg(test)]

use super::*;
use proptest::prelude::*;
use soroban_sdk::testutils::{Address as _, Ledger};
use soroban_sdk::token::{Client as TokenClient, StellarAssetClient};

const PERIOD: u64 = 3_600;
const AMOUNT: i128 = 1_000;

struct Fixture<'a> {
    env: Env,
    client: SubscriptionsClient<'a>,
    token: TokenClient<'a>,
    contract_id: Address,
    merchant: Address,
    subscriber: Address,
}

fn setup<'a>() -> Fixture<'a> {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|l| l.timestamp = 1_000);

    let contract_id = env.register(Subscriptions, ());
    let client = SubscriptionsClient::new(&env, &contract_id);

    let issuer = Address::generate(&env);
    let sac = env.register_stellar_asset_contract_v2(issuer);
    let token = TokenClient::new(&env, &sac.address());

    Fixture {
        client,
        token,
        contract_id,
        merchant: Address::generate(&env),
        subscriber: Address::generate(&env),
        env,
    }
}

impl Fixture<'_> {
    fn fund_and_approve(&self, minted: i128, allowance: i128) {
        StellarAssetClient::new(&self.env, &self.token.address).mint(&self.subscriber, &minted);
        self.token
            .approve(&self.subscriber, &self.contract_id, &allowance, &100_000);
    }

    fn plan(&self) -> u32 {
        self.client
            .create_plan(&self.merchant, &self.token.address, &AMOUNT, &PERIOD)
    }

    fn advance(&self, secs: u64) {
        self.env.ledger().with_mut(|l| l.timestamp += secs);
    }
}

#[test]
fn create_plan_stores_plan() {
    let f = setup();
    let id = f.plan();
    assert_eq!(id, 1);
    let plan = f.client.get_plan(&id);
    assert_eq!(plan.merchant, f.merchant);
    assert_eq!(plan.amount, AMOUNT);
    assert_eq!(plan.period, PERIOD);
    assert!(plan.active);
    assert_eq!(f.client.plan_count(), 1);
}

#[test]
fn create_plan_rejects_bad_inputs() {
    let f = setup();
    let t = &f.token.address;
    assert_eq!(
        f.client.try_create_plan(&f.merchant, t, &0, &PERIOD),
        Err(Ok(Error::InvalidAmount))
    );
    assert_eq!(
        f.client.try_create_plan(&f.merchant, t, &-5, &PERIOD),
        Err(Ok(Error::InvalidAmount))
    );
    assert_eq!(
        f.client
            .try_create_plan(&f.merchant, t, &AMOUNT, &(MIN_PERIOD - 1)),
        Err(Ok(Error::InvalidPeriod))
    );
}

#[test]
fn subscribe_pays_first_period_immediately() {
    let f = setup();
    f.fund_and_approve(10_000, 10_000);
    let plan_id = f.plan();
    let sub_id = f.client.subscribe(&f.subscriber, &plan_id);

    assert_eq!(f.token.balance(&f.merchant), AMOUNT);
    assert_eq!(f.token.balance(&f.subscriber), 10_000 - AMOUNT);
    assert_eq!(f.token.balance(&f.contract_id), 0);

    let sub = f.client.get_subscription(&sub_id);
    assert_eq!(sub.charges, 1);
    assert_eq!(sub.next_charge, 1_000 + PERIOD);
    assert!(!sub.cancelled);
}

#[test]
fn subscribe_without_allowance_fails_and_moves_nothing() {
    let f = setup();
    StellarAssetClient::new(&f.env, &f.token.address).mint(&f.subscriber, &10_000);
    let plan_id = f.plan();
    assert!(f.client.try_subscribe(&f.subscriber, &plan_id).is_err());
    assert_eq!(f.token.balance(&f.subscriber), 10_000);
    assert_eq!(f.token.balance(&f.merchant), 0);
}

#[test]
fn subscribe_unknown_plan_fails() {
    let f = setup();
    assert_eq!(
        f.client.try_subscribe(&f.subscriber, &99),
        Err(Ok(Error::PlanNotFound))
    );
}

#[test]
fn charge_before_due_is_rejected() {
    let f = setup();
    f.fund_and_approve(10_000, 10_000);
    let sub_id = f.client.subscribe(&f.subscriber, &f.plan());
    f.advance(PERIOD - 1);
    assert!(!f.client.is_due(&sub_id));
    assert_eq!(f.client.try_charge(&sub_id), Err(Ok(Error::NotDue)));
}

#[test]
fn charge_when_due_moves_funds_and_reschedules() {
    let f = setup();
    f.fund_and_approve(10_000, 10_000);
    let sub_id = f.client.subscribe(&f.subscriber, &f.plan());
    f.advance(PERIOD);
    assert!(f.client.is_due(&sub_id));
    f.client.charge(&sub_id);

    assert_eq!(f.token.balance(&f.merchant), 2 * AMOUNT);
    let sub = f.client.get_subscription(&sub_id);
    assert_eq!(sub.charges, 2);
    assert_eq!(sub.next_charge, 1_000 + PERIOD + PERIOD);
}

#[test]
fn missed_periods_are_not_back_charged() {
    let f = setup();
    f.fund_and_approve(100_000, 100_000);
    let sub_id = f.client.subscribe(&f.subscriber, &f.plan());
    f.advance(PERIOD * 5);
    f.client.charge(&sub_id);
    assert_eq!(f.client.try_charge(&sub_id), Err(Ok(Error::NotDue)));
    assert_eq!(f.token.balance(&f.merchant), 2 * AMOUNT);
}

#[test]
fn allowance_caps_total_pulled() {
    let f = setup();
    f.fund_and_approve(10_000, AMOUNT); // allowance covers one charge only
    let sub_id = f.client.subscribe(&f.subscriber, &f.plan());
    f.advance(PERIOD);
    assert!(f.client.try_charge(&sub_id).is_err());
    assert_eq!(f.token.balance(&f.merchant), AMOUNT);
}

#[test]
fn cancel_stops_charges_and_cannot_repeat() {
    let f = setup();
    f.fund_and_approve(10_000, 10_000);
    let sub_id = f.client.subscribe(&f.subscriber, &f.plan());
    f.client.cancel(&sub_id);
    assert_eq!(f.env.auths()[0].0, f.subscriber);

    f.advance(PERIOD);
    assert!(!f.client.is_due(&sub_id));
    assert_eq!(
        f.client.try_charge(&sub_id),
        Err(Ok(Error::AlreadyCancelled))
    );
    assert_eq!(
        f.client.try_cancel(&sub_id),
        Err(Ok(Error::AlreadyCancelled))
    );
    assert_eq!(f.token.balance(&f.merchant), AMOUNT);
}

#[test]
fn deactivated_plan_blocks_new_and_existing() {
    let f = setup();
    f.fund_and_approve(10_000, 10_000);
    let plan_id = f.plan();
    let sub_id = f.client.subscribe(&f.subscriber, &plan_id);
    f.client.deactivate_plan(&plan_id);

    f.advance(PERIOD);
    assert_eq!(f.client.try_charge(&sub_id), Err(Ok(Error::PlanInactive)));
    assert_eq!(
        f.client.try_subscribe(&f.subscriber, &plan_id),
        Err(Ok(Error::PlanInactive))
    );
}

#[test]
fn unknown_subscription_errors() {
    let f = setup();
    assert_eq!(
        f.client.try_charge(&7),
        Err(Ok(Error::SubscriptionNotFound))
    );
    assert_eq!(
        f.client.try_cancel(&7),
        Err(Ok(Error::SubscriptionNotFound))
    );
}

#[test]
#[should_panic]
fn create_plan_requires_merchant_auth() {
    let env = Env::default(); // no mock_all_auths
    let id = env.register(Subscriptions, ());
    let client = SubscriptionsClient::new(&env, &id);
    let merchant = Address::generate(&env);
    let token = Address::generate(&env);
    client.create_plan(&merchant, &token, &AMOUNT, &PERIOD);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    /// Conservation: tokens are only ever moved subscriber -> merchant, the
    /// contract never holds a balance, and the merchant receives exactly
    /// `amount * charges`.
    #[test]
    fn funds_are_conserved(amount in 1i128..1_000_000, charges in 1u32..15) {
        let f = setup();
        let total = amount * 20;
        f.fund_and_approve(total, total);
        let plan_id = f.client.create_plan(&f.merchant, &f.token.address, &amount, &PERIOD);
        let sub_id = f.client.subscribe(&f.subscriber, &plan_id);
        for _ in 1..charges {
            f.advance(PERIOD);
            f.client.charge(&sub_id);
        }
        let paid = amount * i128::from(charges);
        prop_assert_eq!(f.token.balance(&f.merchant), paid);
        prop_assert_eq!(f.token.balance(&f.subscriber), total - paid);
        prop_assert_eq!(f.token.balance(&f.contract_id), 0);
        prop_assert_eq!(f.client.get_subscription(&sub_id).charges, charges);
    }
}
