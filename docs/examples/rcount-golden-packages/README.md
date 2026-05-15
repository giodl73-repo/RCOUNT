# RCOUNT Golden Packages

These packages are tiny synthetic election-count fixtures. They are not real
election data and make no legal certification claim.

## Packages

| Package | Generator | Purpose |
|---|---|---|
| `summary-basic` | `cargo run -p rcount-io --example summary_basic_package` | One contest, two precincts, one jurisdiction total; verifies `contest_selection_sum` and `jurisdiction_contest_total`. |
| `canvass-correction` | `cargo run -p rcount-io --example canvass_correction_package` | Unofficial and canvassed snapshots where a public correction event explains the changed total. |
| `mail-batch-added` | `cargo run -p rcount-io --example mail_batch_added_package` | Batched precinct summaries where a late mail batch is declared and reconciled. |
| `precinct-split-lineage` | `cargo run -p rcount-io --example precinct_split_lineage_package` | Cross-cycle reporting-unit lineage: one precinct split and two precincts merged. |
| `privacy-inclusion-sketch` | `cargo run -p rcount-io --example privacy_inclusion_sketch_package` | Receipt-safe inclusion proof: an anonymized token is present without choices. |
| `cvr-summary` | `cargo run -p rcount-io --example cvr_summary_package` | CVR contest rows reconcile to public summary selections and residuals. |
| `rla-replay` | `cargo run -p rcount-io --example rla_replay_package` | Replayable RLA sample: public seed, contest manifest hash, risk limit, and algorithm id reproduce the selected CVR ids. |
| `rla-stopping` | `cargo run -p rcount-io --example rla_stopping_package` | RLA sample observations match CVR interpretations and satisfy the declared stopping threshold. |
| `rla-discrepancy` | `cargo run -p rcount-io --example rla_discrepancy_package` | RLA observation discrepancy is explicitly classified and the declared escalation verifies. |
| `rla-margin` | `cargo run -p rcount-io --example rla_margin_package` | RLA reported winner/loser votes, margin, and denominator are bound to public jurisdiction totals. |
| `rla-statistical` | `cargo run -p rcount-io --example rla_statistical_package` | Named comparison-margin stopping method recomputes declared risk estimate and verifies pass/escalate status. |
| `colorado-rla` | `cargo run -p rcount-io --example colorado_rla_package` | Colorado-style RLA adapter names a jurisdiction method and verifies 20-digit seed, SHA-256 sampler, margin, and stopping fields. |
| `california-rla` | `cargo run -p rcount-io --example california_rla_package` | California-style RLA adapter names public audit software metadata and the required ballot manifest format. |
| `manual-audit` | `cargo run -p rcount-io --example manual_audit_package` | Ordinary manual-audit fixture: hand-count totals match canvassed machine totals within a zero-vote tolerance. |
| `district-aggregation-rplan` | `cargo run -p rcount-district --example district_aggregation_rplan` | Optional RPLAN bridge: verified precinct summaries are assigned into district totals with package and plan hashes. |
| `multi-election-harness` | `cargo run -p rcount-district --example multi_election_harness` | L2 synthetic state: three election cycles with precinct split/merge lineage and per-cycle RPLAN district aggregation. |
| `multi-election-harness-negatives` | `cargo run -p rcount-district --example multi_election_negative_harnesses` | L2 negative cases: bad lineage, stale RPLAN unit assignment, and tampered cycle source evidence. |
| `bad-selection-sum` | `cargo run -p rcount-io --example bad_selection_sum_package` | Negative fixture: manifest and source hashes verify, but local contest arithmetic fails. |
| `missing-batch` | `cargo run -p rcount-io --example missing_batch_package` | Negative fixture: manifest and source hashes verify, but a batch summary references absent batch evidence. |
| `bad-lineage` | `cargo run -p rcount-io --example bad_lineage_package` | Negative fixture: manifest and source hashes verify, but lineage references a missing current unit. |
| `choice-bearing-proof` | `cargo run -p rcount-io --example choice_bearing_proof_package` | Negative fixture: proof reveals a candidate selection and fails the privacy gate. |
| `bad-cvr-summary` | `cargo run -p rcount-io --example bad_cvr_summary_package` | Negative fixture: CVR rows no longer reconcile to the public summary. |
| `bad-rla-replay` | `cargo run -p rcount-io --example bad_rla_replay_package` | Negative fixture: the published RLA sample draw no longer replays from the seed and contest manifest hash. |
| `bad-rla-stopping` | `cargo run -p rcount-io --example bad_rla_stopping_package` | Negative fixture: the sample replays, but observed marks imply escalation rather than declared pass. |
| `bad-rla-discrepancy` | `cargo run -p rcount-io --example bad_rla_discrepancy_package` | Negative fixture: observed marks imply one discrepancy type, but the declared taxonomy names another. |
| `bad-rla-margin` | `cargo run -p rcount-io --example bad_rla_margin_package` | Negative fixture: reported margin metadata drifts from the public jurisdiction total. |
| `bad-rla-statistical` | `cargo run -p rcount-io --example bad_rla_statistical_package` | Negative fixture: declared risk estimate no longer matches the named stopping method. |
| `bad-colorado-rla` | `cargo run -p rcount-io --example bad_colorado_rla_package` | Negative fixture: Colorado-style adapter rejects a public seed that is not 20 decimal digits. |
| `bad-california-rla` | `cargo run -p rcount-io --example bad_california_rla_package` | Negative fixture: California-style adapter rejects audit software source metadata that is not a public URL. |
| `bad-manual-audit` | `cargo run -p rcount-io --example bad_manual_audit_package` | Negative fixture: hand-count totals exceed the declared tolerance while the audit declares pass. |
| `tampered-source` | copied from `summary-basic`, then raw source bytes edited | Negative fixture: arithmetic still passes, but `source_hash_match` fails. |
| `missing-source-hash` | copied from `summary-basic`, then source index emptied | Negative fixture: package records omit the raw source hash evidence. |

The verifier surface is still crate-level while RCOUNT is incubating. The first
fixtures are generated from `rcount_core` synthetic packages and written
through `rcount-io`.

The real verifier transcript is generated with:

```text
cargo run -p rcount-audit --example write_summary_basic_transcript
cargo run -p rcount-cli -- verify docs/examples/rcount-golden-packages/canvass-correction --write-transcript
```

The CLI verifier can check the package directly:

```text
cargo run -p rcount-cli -- verify docs/examples/rcount-golden-packages/summary-basic
cargo run -p rcount-cli -- verify docs/examples/rcount-golden-packages/canvass-correction
cargo run -p rcount-cli -- verify docs/examples/rcount-golden-packages/mail-batch-added
cargo run -p rcount-cli -- verify docs/examples/rcount-golden-packages/precinct-split-lineage
cargo run -p rcount-cli -- verify docs/examples/rcount-golden-packages/privacy-inclusion-sketch
cargo run -p rcount-cli -- verify docs/examples/rcount-golden-packages/cvr-summary
cargo run -p rcount-cli -- verify docs/examples/rcount-golden-packages/rla-replay
cargo run -p rcount-cli -- verify docs/examples/rcount-golden-packages/rla-stopping
cargo run -p rcount-cli -- verify docs/examples/rcount-golden-packages/rla-discrepancy
cargo run -p rcount-cli -- verify docs/examples/rcount-golden-packages/rla-margin
cargo run -p rcount-cli -- verify docs/examples/rcount-golden-packages/rla-statistical
cargo run -p rcount-cli -- verify docs/examples/rcount-golden-packages/colorado-rla
cargo run -p rcount-cli -- verify docs/examples/rcount-golden-packages/california-rla
cargo run -p rcount-cli -- verify docs/examples/rcount-golden-packages/manual-audit
cargo run -p rcount-cli -- aggregate-districts docs/examples/rcount-golden-packages/district-aggregation-rplan/package --plan docs/examples/rcount-golden-packages/district-aggregation-rplan/plan.rplan.json
cargo run -p rcount-district --example multi_election_harness
cargo run -p rcount-district --example multi_election_negative_harnesses
cargo run -p rcount-cli -- verify docs/examples/rcount-golden-packages/bad-selection-sum
cargo run -p rcount-cli -- verify docs/examples/rcount-golden-packages/missing-batch
cargo run -p rcount-cli -- verify docs/examples/rcount-golden-packages/bad-lineage
cargo run -p rcount-cli -- verify docs/examples/rcount-golden-packages/choice-bearing-proof
cargo run -p rcount-cli -- verify docs/examples/rcount-golden-packages/bad-cvr-summary
cargo run -p rcount-cli -- verify docs/examples/rcount-golden-packages/bad-rla-replay
cargo run -p rcount-cli -- verify docs/examples/rcount-golden-packages/bad-rla-stopping
cargo run -p rcount-cli -- verify docs/examples/rcount-golden-packages/bad-rla-discrepancy
cargo run -p rcount-cli -- verify docs/examples/rcount-golden-packages/bad-rla-margin
cargo run -p rcount-cli -- verify docs/examples/rcount-golden-packages/bad-rla-statistical
cargo run -p rcount-cli -- verify docs/examples/rcount-golden-packages/bad-colorado-rla
cargo run -p rcount-cli -- verify docs/examples/rcount-golden-packages/bad-california-rla
cargo run -p rcount-cli -- verify docs/examples/rcount-golden-packages/bad-manual-audit
cargo run -p rcount-cli -- verify docs/examples/rcount-golden-packages/tampered-source
cargo run -p rcount-cli -- verify docs/examples/rcount-golden-packages/missing-source-hash
cargo run -p rcount-cli -- verify docs/examples/rcount-golden-packages/multi-election-harness-negatives/bad-lineage/SYN-2028-general/package
cargo run -p rcount-cli -- aggregate-districts docs/examples/rcount-golden-packages/multi-election-harness-negatives/stale-plan/SYN-2028-general/package --plan docs/examples/rcount-golden-packages/multi-election-harness-negatives/stale-plan/SYN-2028-general/plan.rplan.json --contest-id syn-cycle-mayor
cargo run -p rcount-cli -- verify docs/examples/rcount-golden-packages/multi-election-harness-negatives/tampered-2028-source/SYN-2028-general/package
```
