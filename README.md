# stellar-subscriptions

Recurring **pull payments** for any SEP-41 token on Stellar, written as a Soroban smart contract in Rust.

A merchant publishes a plan (token, amount, period). A subscriber approves the contract as a token spender and subscribes; the first period is paid immediately. Later periods can be charged by **anyone** (a keeper bot) once due. Funds go straight `subscriber → merchant`; the contract never custodies user funds, the subscriber can cancel at any time, and the token allowance is the hard cap on what can ever be pulled.

> Status: pre-testnet. See [docs/FEATURE-STATUS.md](docs/FEATURE-STATUS.md) for exactly what is and is not verified, and [docs/KNOWN-LIMITATIONS.md](docs/KNOWN-LIMITATIONS.md) for what it deliberately does not do.

## Proof at a glance

| Item | Value |
|---|---|
| Network | Stellar Testnet |
| Contract ID | _fill in after `scripts/demo-testnet.sh`_ |
| WASM sha256 | _fill in (also attached to each GitHub Release)_ |
| Deploy tx | _link_ |
| First charge tx | _link_ |
| CI | ![CI](https://github.com/Alimzy/stellar-subscriptions/actions/workflows/ci.yml/badge.svg) |

## Interface

| Function | Auth | Purpose |
|---|---|---|
| `create_plan(merchant, token, amount, period) -> plan_id` | merchant | Publish a plan |
| `deactivate_plan(plan_id)` | merchant | Block new subscribers and further charges |
| `subscribe(subscriber, plan_id) -> sub_id` | subscriber | Subscribe, pay first period now |
| `charge(sub_id)` | none (keeper) | Pull one due period |
| `cancel(sub_id)` | subscriber | Stop the subscription |
| `get_plan`, `get_subscription`, `is_due`, `plan_count`, `subscription_count` | none | Views |

Events: `PlanCreated`, `PlanDeactivated`, `Subscribed`, `Charged`, `Cancelled`.

## Develop

```bash
rustup target add wasm32v1-none
cargo generate-lockfile   # commit Cargo.lock; CI uses --locked
make check                # fmt + clippy + tests + wasm build
```

## Try it on testnet

```bash
DRY_RUN=1 bash scripts/demo-testnet.sh   # prints every command
bash scripts/demo-testnet.sh             # needs stellar-cli and curl
```

## Docs

[Architecture](docs/ARCHITECTURE.md) · [Feature status](docs/FEATURE-STATUS.md) · [Known limitations](docs/KNOWN-LIMITATIONS.md) · [Contributing](CONTRIBUTING.md) · [Security](SECURITY.md)

## License

MIT
