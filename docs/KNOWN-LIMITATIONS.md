# Known limitations

- **Unaudited.** Do not use with real funds.
- **No back-charging.** If a keeper is late, missed periods are skipped, not collected later.
- **Fixed price per plan.** To change the price, create a new plan.
- **No trials, proration, refunds or free periods.**
- **Allowance is the subscriber's job.** If it is exhausted or expired, `charge` fails; there is no on-chain retry or dunning.
- **No keeper incentive.** Anyone can call `charge`, but nothing pays them to.
- **No pause.** Subscribers can only cancel; merchants can only deactivate the whole plan.
- **Per-ledger-time precision.** Timing uses ledger timestamps, not wall-clock guarantees.
- **Non-standard tokens** (fee-on-transfer, rebasing) are untested.
- **Unmaintained transitive dependency.** `cargo deny` ignores RUSTSEC-2024-0436 (`paste` is unmaintained, not vulnerable). It comes in through `soroban-sdk` and we cannot upgrade it ourselves. The ignore is in `deny.toml` with its reason, and should be removed once the SDK drops it.
