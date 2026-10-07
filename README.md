# stellar-subscriptions

Recurring **pull payments** for any SEP-41 token on Stellar, written as a Soroban smart contract in Rust.

A merchant publishes a plan (token, amount, period). A subscriber approves the contract as a token spender and subscribes; the first period is paid immediately. Later periods can be charged by **anyone** (a keeper bot) once due. Funds go straight `subscriber → merchant`; the contract never custodies user funds, the subscriber can cancel at any time, and the token allowance is the hard cap on what can ever be pulled.

> Status: pre-testnet. See [docs/FEATURE-STATUS.md](docs/FEATURE-STATUS.md) for exactly what is and is not verified, and [docs/KNOWN-LIMITATIONS.md](docs/KNOWN-LIMITATIONS.md) for what it deliberately does not do.

## Proof at a glance

| Item | Value |
|---|---|
| Network | Stellar Testnet |
| Contract ID | [`CAEBLJLKCAMVZ6SWCYYZID4HHJJTUZZ4PP4OC4UDPRP6XEH22NCOMQLR`](https://lab.stellar.org/r/testnet/contract/CAEBLJLKCAMVZ6SWCYYZID4HHJJTUZZ4PP4OC4UDPRP6XEH22NCOMQLR) |
| WASM sha256 | `8669f9584646e871829cdb3ae96af89ec27a73e84319b868c1526faaa63feaa0` |
| WASM source | [GitHub Release v0.1.0](https://github.com/Alimzy/stellar-subscriptions/releases/tag/v0.1.0) (`SHA256SUMS`), built by CI and deployed unchanged |
| WASM size | 9,559 bytes (cap 65,536) |
| WASM upload | [`810a463fda…`](https://stellar.expert/explorer/testnet/tx/810a463fdac7cca703839ef1753285502f182960e91de983a679ed7e296c8cb6) |
| Deploy | [`10e18a52f7…`](https://stellar.expert/explorer/testnet/tx/10e18a52f753dabf73d57ba2cc56cc565f999448a0f016e2fc38bb82bf8d8616) |
| `approve` | [`e2af70e446…`](https://stellar.expert/explorer/testnet/tx/e2af70e446928822fd8880474da16e2e30a9ade3f2943d7293c55ecf26c45a2a) |
| `create_plan` | [`3094642d4a…`](https://stellar.expert/explorer/testnet/tx/3094642d4a1f6861ecedefe8179185a3a6ebf6a71ea69ef3a24806a2091eb2ab) |
| `subscribe` (1 XLM paid) | [`8308aaaf27…`](https://stellar.expert/explorer/testnet/tx/8308aaaf272f5ecbca1a3c2888e9326ecc877a5ec1aa22899e8515b0bd4cb5eb) |
| `charge` (2nd period, 1 XLM paid) | [`fcb379629a…`](https://stellar.expert/explorer/testnet/tx/fcb379629aadbd0cf16f0e4a5bbc6d7be2abb4554c63d37c36dafac2a8d9f483) |
| CI | ![CI](https://github.com/Alimzy/stellar-subscriptions/actions/workflows/ci.yml/badge.svg) |

The `charge` transaction emits a `Charged` event (charges = 2) and a token transfer from subscriber to merchant.

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
