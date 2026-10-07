# Feature status

Legend: ✅ verified · 🟡 written, not yet verified · ⛔ not implemented

CI run: [37581788517](https://github.com/Alimzy/stellar-subscriptions/actions/runs/37581788517) (fmt, clippy `-D warnings`, 14 tests incl. property test, WASM build + size cap, docs, audit/deny all green)
Testnet: contract `CCVC2FX7Q3IFIGNXLLLSRGXWV7JKCNYLMXWKQIXVNIZTSGU7QFVJSHIK` (transactions in the README proof table)

| Feature | Status | Evidence |
|---|---|---|
| create / deactivate plan | ✅ | unit tests; `create_plan` on testnet |
| subscribe with immediate first charge via `transfer_from` | ✅ | unit tests; testnet `subscribe` moved 1 XLM |
| keeper-callable `charge`, no back-charging | ✅ | unit tests; testnet `charge` (charges = 2) |
| cancel | ✅ | unit tests (not run on testnet) |
| Events | ✅ | observed on testnet; not asserted in unit tests |
| TTL bumps | 🟡 | implemented, no test checks them |
| Property test (funds conserved, contract balance 0) | ✅ | CI `test` job |
| Testnet deployment | ✅ | README proof table |
| External security audit | ⛔ | none |
| Mainnet | ⛔ | none |
