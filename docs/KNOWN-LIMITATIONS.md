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
