# Architecture

## Flow
1. Merchant: `create_plan`.
2. Subscriber: `token.approve(subscriber, <this contract>, allowance, expiration_ledger)` then `subscribe`.
3. Keeper (anyone): `charge(sub_id)` once `now >= next_charge`.
4. Subscriber: `cancel(sub_id)` any time; or let the allowance expire.

## Why `transfer_from`
The contract is the *spender*. It can never take more than the allowance the subscriber granted, and never holds a balance. A property test asserts the contract balance stays 0.

## Storage
| Key | Storage | Value |
|---|---|---|
| `PlanCount`, `SubCount` | instance | `u32` counters |
| `Plan(id)` | persistent | `Plan` |
| `Sub(id)` | persistent | `Subscription` |

Every write bumps TTL (threshold 29 days, extend to 30 days). Views do not bump.

## Ordering
State is written before the token call. Soroban forbids contract re-entrancy and any failure reverts the whole transaction, so there is no partial state.

## Timing
`charge` sets `next_charge = now + period` (no back-charging of missed periods).
