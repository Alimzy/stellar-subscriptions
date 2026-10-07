# stellar-subscriptions

Recurring **pull payments** for any SEP-41 token on Stellar, written as a Soroban smart contract in Rust.

A merchant publishes a plan (token, amount, period). A subscriber approves the contract as a token spender and subscribes; the first period is paid immediately. Later periods can be charged by **anyone** (a keeper bot) once due. Funds go straight `subscriber → merchant`; the contract never custodies user funds, the subscriber can cancel at any time, and the token allowance is the hard cap on what can ever be pulled.

> Status: pre-testnet. See [docs/FEATURE-STATUS.md](docs/FEATURE-STATUS.md) for exactly what is and is not verified, and [docs/KNOWN-LIMITATIONS.md](docs/KNOWN-LIMITATIONS.md) for what it deliberately does not do.

## Proof at a glance

| Item | Value |
|---|---|
| Network | Stellar Testnet |
| Contract ID | [`CCVC2FX7Q3IFIGNXLLLSRGXWV7JKCNYLMXWKQIXVNIZTSGU7QFVJSHIK`](https://lab.stellar.org/r/testnet/contract/CCVC2FX7Q3IFIGNXLLLSRGXWV7JKCNYLMXWKQIXVNIZTSGU7QFVJSHIK) |
| WASM sha256 | `ad78763d6a0fd8aa777f1496ba1a2f0b6ce7625d26511304ed58d083880c3ed2` |
| WASM size | 9,559 bytes |
| Deploy | [`20ee590714…`](https://stellar.expert/explorer/testnet/tx/20ee590714256507ce68e67f639cb90aff3b442b0edd7eac6b3a659bcb8d7fed) |
| `approve` | [`52bbb169f9…`](https://stellar.expert/explorer/testnet/tx/52bbb169f9641c4875267306ceed2d2befbb4884e2bdb9b9acf3a739860bbf4c) |
| `create_plan` | [`289e13ca74…`](https://stellar.expert/explorer/testnet/tx/289e13ca741e4c76e7cbd17e94240ea144cb88ae20eb960c39623659d250b981) |
| `subscribe` (1 XLM paid) | [`9496e518c9…`](https://stellar.expert/explorer/testnet/tx/9496e518c9484b81dacdfce1dc795cf132e8ff36f6dfb7e1b9193cac5e0497fe) |
| `charge` (2nd period, 1 XLM paid) | [`7b729bac56…`](https://stellar.expert/explorer/testnet/tx/7b729bac567b965018c3f36593d1f4b7574ffc2d68b8ba9a8f1b0cae116b0a35) |
| CI | ![CI](https://github.com/Alimzy/stellar-subscriptions/actions/workflows/ci.yml/badge.svg) |

Read back after the second charge: `charges: 2`, `cancelled: false`.

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
