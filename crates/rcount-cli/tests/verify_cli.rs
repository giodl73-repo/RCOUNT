use std::process::Command;

use rcount_core::{
    AuditAlgorithmDecision, AuditAlgorithmRun, AuditAssertion, AuditAssertionKind, AuditSampleStep,
    AuditSamplingMode, RationalValue, ALPHA_MARTINGALE_METHOD_ID, BATCH_COMPARISON_METHOD_ID,
    BRAVO_BALLOT_POLLING_METHOD_ID, KAPLAN_MARKOV_COMPARISON_METHOD_ID,
};
use rcount_io::{synthetic_summary_basic_manifest, write_package_dir};

fn docs_summary_basic_path() -> String {
    docs_package_path("summary-basic")
}

fn docs_canvass_correction_path() -> String {
    docs_package_path("canvass-correction")
}

fn docs_bad_selection_sum_path() -> String {
    docs_package_path("bad-selection-sum")
}

fn docs_mail_batch_added_path() -> String {
    docs_package_path("mail-batch-added")
}

fn docs_missing_batch_path() -> String {
    docs_package_path("missing-batch")
}

fn docs_precinct_split_lineage_path() -> String {
    docs_package_path("precinct-split-lineage")
}

fn docs_bad_lineage_path() -> String {
    docs_package_path("bad-lineage")
}

fn docs_privacy_inclusion_sketch_path() -> String {
    docs_package_path("privacy-inclusion-sketch")
}

fn docs_choice_bearing_proof_path() -> String {
    docs_package_path("choice-bearing-proof")
}

fn docs_cvr_summary_path() -> String {
    docs_package_path("cvr-summary")
}

fn docs_bad_cvr_summary_path() -> String {
    docs_package_path("bad-cvr-summary")
}

fn docs_rla_replay_path() -> String {
    docs_package_path("rla-replay")
}

fn docs_bad_rla_replay_path() -> String {
    docs_package_path("bad-rla-replay")
}

fn docs_rla_stopping_path() -> String {
    docs_package_path("rla-stopping")
}

fn docs_bad_rla_stopping_path() -> String {
    docs_package_path("bad-rla-stopping")
}

fn docs_rla_discrepancy_path() -> String {
    docs_package_path("rla-discrepancy")
}

fn docs_bad_rla_discrepancy_path() -> String {
    docs_package_path("bad-rla-discrepancy")
}

fn docs_rla_margin_path() -> String {
    docs_package_path("rla-margin")
}

fn docs_bad_rla_margin_path() -> String {
    docs_package_path("bad-rla-margin")
}

fn docs_rla_statistical_path() -> String {
    docs_package_path("rla-statistical")
}

fn docs_bad_rla_statistical_path() -> String {
    docs_package_path("bad-rla-statistical")
}

fn docs_colorado_rla_path() -> String {
    docs_package_path("colorado-rla")
}

fn docs_bad_colorado_rla_path() -> String {
    docs_package_path("bad-colorado-rla")
}

fn docs_california_rla_path() -> String {
    docs_package_path("california-rla")
}

fn docs_bad_california_rla_path() -> String {
    docs_package_path("bad-california-rla")
}

fn docs_manual_audit_path() -> String {
    docs_package_path("manual-audit")
}

fn docs_bad_manual_audit_path() -> String {
    docs_package_path("bad-manual-audit")
}

fn docs_district_aggregation_dir() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap()
        .join("docs/examples/rcount-golden-packages")
        .join("district-aggregation-rplan")
}

fn docs_multi_election_cycle_path(cycle_id: &str) -> String {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap()
        .join("docs/examples/rcount-golden-packages")
        .join("multi-election-harness")
        .join(cycle_id)
        .join("package")
        .to_string_lossy()
        .into_owned()
}

fn docs_multi_election_negative_path(case_name: &str, cycle_id: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap()
        .join("docs/examples/rcount-golden-packages")
        .join("multi-election-harness-negatives")
        .join(case_name)
        .join(cycle_id)
}

fn docs_package_path(package_name: &str) -> String {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap()
        .join("docs/examples/rcount-golden-packages")
        .join(package_name)
        .to_string_lossy()
        .into_owned()
}

#[test]
fn verify_summary_basic_exits_zero() {
    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .args(["verify", &docs_summary_basic_path(), "--format", "json"])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""status":"pass""#));
    assert!(stdout.contains(r#""equation_id":"contest_selection_sum""#));
}

#[test]
fn import_statement_csv_then_verify_exits_zero() {
    let tmp = tempfile::tempdir().unwrap();
    let csv_path = tmp.path().join("statement.csv");
    std::fs::write(
        &csv_path,
        concat!(
            "contest_id,contest_title,vote_for,selection_id,selection_label,selection_kind,reporting_unit_id,reporting_unit_kind,parent_jurisdiction,status,votes,undervotes,overvotes,blank_contests,counted_ballots\n",
            "syn-2024-mayor,Synthetic Mayor,1,cand-a,Candidate A,candidate,syn:precinct:P-001,precinct,syn-county-1,canvassed,40,3,1,0,80\n",
            "syn-2024-mayor,Synthetic Mayor,1,cand-b,Candidate B,candidate,syn:precinct:P-001,precinct,syn-county-1,canvassed,35,3,1,0,80\n",
            "syn-2024-mayor,Synthetic Mayor,1,write-in,Write-in,write-in-bucket,syn:precinct:P-001,precinct,syn-county-1,canvassed,1,3,1,0,80\n",
            "syn-2024-mayor,Synthetic Mayor,1,cand-a,Candidate A,candidate,syn:precinct:P-002,precinct,syn-county-1,canvassed,25,4,0,1,60\n",
            "syn-2024-mayor,Synthetic Mayor,1,cand-b,Candidate B,candidate,syn:precinct:P-002,precinct,syn-county-1,canvassed,30,4,0,1,60\n",
            "syn-2024-mayor,Synthetic Mayor,1,write-in,Write-in,write-in-bucket,syn:precinct:P-002,precinct,syn-county-1,canvassed,0,4,0,1,60\n",
            "syn-2024-mayor,Synthetic Mayor,1,cand-a,Candidate A,candidate,syn:jurisdiction:SYN,jurisdiction-total,syn,canvassed,65,7,1,1,140\n",
            "syn-2024-mayor,Synthetic Mayor,1,cand-b,Candidate B,candidate,syn:jurisdiction:SYN,jurisdiction-total,syn,canvassed,65,7,1,1,140\n",
            "syn-2024-mayor,Synthetic Mayor,1,write-in,Write-in,write-in-bucket,syn:jurisdiction:SYN,jurisdiction-total,syn,canvassed,1,7,1,1,140\n",
        ),
    )
    .unwrap();
    let package_dir = tmp.path().join("package");

    let import = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .arg("import-statement-csv")
        .arg(&csv_path)
        .arg(&package_dir)
        .output()
        .unwrap();
    assert!(
        import.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&import.stderr)
    );

    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .arg("verify")
        .arg(&package_dir)
        .args(["--format", "json"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""reporting_unit_id":"source:statement-csv""#));
    assert!(stdout.contains(r#""equation_id":"jurisdiction_contest_total""#));
}

#[test]
fn import_nist_cdf_json_then_verify_exits_zero() {
    let tmp = tempfile::tempdir().unwrap();
    let json_path = tmp.path().join("cdf.json");
    std::fs::write(
        &json_path,
        r#"{
  "ElectionReport": {
    "ResultsStatus": "canvassed",
    "GpUnit": [
      {"@id": "syn:precinct:P-001", "Type": "precinct"},
      {"@id": "syn:precinct:P-002", "Type": "precinct"},
      {"@id": "syn:jurisdiction:SYN", "Type": "county"}
    ],
    "Election": [{
      "Contest": [{
        "@id": "syn-2024-mayor",
        "Name": {"Text": [{"Value": "Synthetic Mayor"}]},
        "NumberElected": 1,
        "ContestSelection": [
          {"@id": "cand-a", "Name": {"Text": [{"Value": "Candidate A"}]}, "VoteCounts": [
            {"GpUnitId": "syn:precinct:P-001", "Count": 40},
            {"GpUnitId": "syn:precinct:P-002", "Count": 25},
            {"GpUnitId": "syn:jurisdiction:SYN", "Count": 65}
          ]},
          {"@id": "cand-b", "Name": {"Text": [{"Value": "Candidate B"}]}, "VoteCounts": [
            {"GpUnitId": "syn:precinct:P-001", "Count": 35},
            {"GpUnitId": "syn:precinct:P-002", "Count": 30},
            {"GpUnitId": "syn:jurisdiction:SYN", "Count": 65}
          ]},
          {"@id": "write-in", "Name": {"Text": [{"Value": "Write-in"}]}, "IsWriteIn": true, "VoteCounts": [
            {"GpUnitId": "syn:precinct:P-001", "Count": 1},
            {"GpUnitId": "syn:precinct:P-002", "Count": 0},
            {"GpUnitId": "syn:jurisdiction:SYN", "Count": 1}
          ]}
        ],
        "OtherCounts": [
          {"GpUnitId": "syn:precinct:P-001", "Undervotes": 3, "Overvotes": 1, "BlankVotes": 0},
          {"GpUnitId": "syn:precinct:P-002", "Undervotes": 4, "Overvotes": 0, "BlankVotes": 1},
          {"GpUnitId": "syn:jurisdiction:SYN", "Undervotes": 7, "Overvotes": 1, "BlankVotes": 1}
        ]
      }]
    }]
  }
}"#,
    )
    .unwrap();
    let package_dir = tmp.path().join("package");

    let import = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .arg("import-nist-cdf-json")
        .arg(&json_path)
        .arg(&package_dir)
        .output()
        .unwrap();
    assert!(
        import.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&import.stderr)
    );

    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .arg("verify")
        .arg(&package_dir)
        .args(["--format", "json"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""reporting_unit_id":"source:nist-cdf-json""#));
    assert!(stdout.contains(r#""equation_id":"jurisdiction_contest_total""#));
}

#[test]
fn import_ri_2024_rep28_rla_then_verify_exits_zero() {
    let tmp = tempfile::tempdir().unwrap();
    let audit_path = tmp.path().join("audit-report.csv");
    let manifest_path = tmp.path().join("manifest.csv");
    let retrieval_path = tmp.path().join("retrieval.csv");
    std::fs::write(
        &audit_path,
        concat!(
            "######## CONTESTS ########,,,,,,,,\n",
            "Contest Name,Targeted?,Number of Winners,Votes Allowed,Total Ballots Cast,Vote Totals,,,\n",
            "Representative 28,Targeted,1,1,10,Scott Guthrie: 4; George A. Nardone: 3; Write-in: 0,,,\n",
            ",,,,,,,,\n",
            "######## AUDIT SETTINGS ########,,,,,,,,\n",
            "Audit Name,Audit Type,Audit Math Type,Risk Limit,Random Seed,Online Data Entry?,,,\n",
            "11-5-24 Representative 28 Ballot Polling Audit,BALLOT_POLLING,MINERVA,9%,34053800000000000000,No,,,\n",
            ",,,,,,,,\n",
            "######## ROUNDS ########,,,,,,,,\n",
            "Round Number,Status,Started At,Sample Size,Risk Measurements,,,\n",
            "1,Ended,2024-11-20 21:52:00+00:00,1,George A. Nardone / Scott Guthrie: 0.054,,,\n",
            ",,,,,,,,\n",
            "######## SAMPLED BALLOTS ########,,,,,,,,\n",
            "Draw Number,Container,Tabulator,Batch Name,Ballot Position,Ticket Numbers,Audit Result,,\n",
            "1,0600,0315412524,EV Coventry,1,Round 1: 0.1,George A. Nardone,,\n",
        ),
    )
    .unwrap();
    std::fs::write(
        &manifest_path,
        concat!(
            "Batch Name,Number of Ballots,Container,Tabulator\n",
            "EV Coventry,6,0600,0315412524\n",
            "MB Coventry 1,4,C0017,8520060462\n",
        ),
    )
    .unwrap();
    std::fs::write(
        &retrieval_path,
        concat!(
            "Container,Tabulator,Batch Name,Ballot Number,Ticket Numbers,Already Audited,Audit Board\n",
            "0600,0315412524,EV Coventry,1,0.1,N,Audit Board #1\n",
        ),
    )
    .unwrap();
    let package_dir = tmp.path().join("package");

    let import = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .arg("import-ri2024-rep28-rla")
        .arg(&audit_path)
        .arg(&manifest_path)
        .arg(&retrieval_path)
        .arg(&package_dir)
        .output()
        .unwrap();
    assert!(
        import.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&import.stderr)
    );

    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .arg("verify")
        .arg(&package_dir)
        .args(["--format", "json"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""status":"pass""#));
    assert!(stdout.contains(r#""reporting_unit_id":"source:ri-rla-ballot-retrieval""#));
    assert!(stdout.contains(r#""equation_id":"accepted_ballots""#));
    assert!(package_dir
        .join("transcripts/ri-2024-rep28-source-summary.json")
        .exists());

    let replay = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .arg("replay-audit-algorithms")
        .arg(&package_dir)
        .args(["--format", "json"])
        .output()
        .unwrap();
    assert_eq!(replay.status.code(), Some(0));
    let replay_stdout = String::from_utf8(replay.stdout).unwrap();
    assert!(replay_stdout.contains(r#""method_id":"minerva-ballot-polling-v1""#));
    assert!(replay_stdout.contains(r#""status":"boundary""#));
    assert!(replay_stdout.contains("Minerva round-one replay requires at least one sample step"));
}

#[test]
fn verify_canvass_correction_exposes_event_correlation() {
    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .args([
            "verify",
            &docs_canvass_correction_path(),
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""status":"pass""#));
    assert!(stdout.contains(r#""equation_id":"canvass_correction_event""#));
}

#[test]
fn verify_bad_selection_sum_exits_one_after_package_read() {
    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .args(["verify", &docs_bad_selection_sum_path(), "--format", "json"])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""equation_id":"contest_selection_sum""#));
    assert!(stdout.contains("contest selection sum mismatch"));
    assert!(stdout.contains(r#""equation_id":"source_hash_match","status":"pass""#));
}

#[test]
fn verify_mail_batch_added_exposes_batch_correlation() {
    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .args(["verify", &docs_mail_batch_added_path(), "--format", "json"])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""equation_id":"batch_summary_total""#));
    assert!(stdout.contains(r#""reporting_unit_id":"batch:P-001:late-mail""#));
}

#[test]
fn verify_missing_batch_exits_one_after_package_read() {
    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .args(["verify", &docs_missing_batch_path(), "--format", "json"])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""equation_id":"batch_summary_total""#));
    assert!(stdout.contains("references missing batch id"));
    assert!(stdout.contains(r#""equation_id":"source_hash_match","status":"pass""#));
}

#[test]
fn verify_precinct_split_lineage_exposes_lineage_correlation() {
    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .args([
            "verify",
            &docs_precinct_split_lineage_path(),
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""equation_id":"lineage_conservation""#));
    assert!(stdout.contains(r#""reporting_unit_id":"lineage:P-004-split""#));
}

#[test]
fn verify_bad_lineage_exits_one_after_package_read() {
    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .args(["verify", &docs_bad_lineage_path(), "--format", "json"])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""equation_id":"lineage_conservation""#));
    assert!(stdout.contains("missing current reporting unit"));
    assert!(stdout.contains(r#""equation_id":"source_hash_match","status":"pass""#));
}

#[test]
fn verify_privacy_inclusion_exposes_privacy_gate() {
    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .args([
            "verify",
            &docs_privacy_inclusion_sketch_path(),
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""equation_id":"proof_privacy_gate""#));
    assert!(stdout.contains(r#""reporting_unit_id":"proof:accepted-token-001""#));
}

#[test]
fn verify_choice_bearing_proof_exits_one_after_package_read() {
    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .args([
            "verify",
            &docs_choice_bearing_proof_path(),
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""equation_id":"proof_privacy_gate""#));
    assert!(stdout.contains("exposes candidate selections"));
    assert!(stdout.contains(r#""equation_id":"source_hash_match","status":"pass""#));
}

#[test]
fn verify_cvr_summary_exposes_cvr_reconciliation() {
    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .args(["verify", &docs_cvr_summary_path(), "--format", "json"])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""equation_id":"cvr_summary_total""#));
    assert!(stdout.contains(r#""reporting_unit_id":"syn:precinct:P-001""#));
}

#[test]
fn verify_bad_cvr_summary_exits_one_after_package_read() {
    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .args(["verify", &docs_bad_cvr_summary_path(), "--format", "json"])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""equation_id":"cvr_summary_total""#));
    assert!(stdout.contains("CVR summary mismatch"));
    assert!(stdout.contains(r#""equation_id":"source_hash_match","status":"pass""#));
}

#[test]
fn verify_rla_replay_exposes_sampler_replay() {
    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .args(["verify", &docs_rla_replay_path(), "--format", "json"])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""equation_id":"rla_sampler_replay""#));
    assert!(stdout.contains(r#""reporting_unit_id":"rla:syn-2024-mayor:round-1""#));
}

#[test]
fn verify_bad_rla_replay_exits_one_after_package_read() {
    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .args(["verify", &docs_bad_rla_replay_path(), "--format", "json"])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""equation_id":"rla_sampler_replay""#));
    assert!(stdout.contains("RLA audit"));
    assert!(stdout.contains("sample mismatch"));
    assert!(stdout.contains(r#""equation_id":"source_hash_match","status":"pass""#));
}

#[test]
fn verify_rla_stopping_exposes_stopping_rule() {
    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .args(["verify", &docs_rla_stopping_path(), "--format", "json"])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""equation_id":"rla_stopping_rule""#));
    assert!(stdout.contains(r#""equation_id":"rla_sampler_replay""#));
}

#[test]
fn verify_bad_rla_stopping_exits_one_after_package_read() {
    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .args(["verify", &docs_bad_rla_stopping_path(), "--format", "json"])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""equation_id":"rla_stopping_rule""#));
    assert!(stdout.contains("declares status Pass, computed Escalate"));
    assert!(stdout.contains(r#""equation_id":"source_hash_match","status":"pass""#));
}

#[test]
fn verify_rla_discrepancy_exposes_taxonomy() {
    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .args(["verify", &docs_rla_discrepancy_path(), "--format", "json"])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""equation_id":"rla_stopping_rule""#));
    assert!(stdout.contains(r#""status":"pass""#));
}

#[test]
fn verify_bad_rla_discrepancy_exits_one_after_package_read() {
    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .args([
            "verify",
            &docs_bad_rla_discrepancy_path(),
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""equation_id":"rla_stopping_rule""#));
    assert!(stdout.contains("discrepancy mismatch"));
    assert!(stdout.contains(r#""equation_id":"source_hash_match","status":"pass""#));
}

#[test]
fn verify_rla_margin_exposes_margin_metadata() {
    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .args(["verify", &docs_rla_margin_path(), "--format", "json"])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""equation_id":"rla_margin_metadata""#));
    assert!(stdout.contains(r#""equation_id":"rla_stopping_rule""#));
}

#[test]
fn verify_bad_rla_margin_exits_one_after_package_read() {
    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .args(["verify", &docs_bad_rla_margin_path(), "--format", "json"])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""equation_id":"rla_margin_metadata""#));
    assert!(stdout.contains("reported margin mismatch"));
    assert!(stdout.contains(r#""equation_id":"source_hash_match","status":"pass""#));
}

#[test]
fn verify_rla_statistical_exposes_risk_estimate() {
    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .args(["verify", &docs_rla_statistical_path(), "--format", "json"])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""equation_id":"rla_margin_metadata""#));
    assert!(stdout.contains(r#""equation_id":"rla_stopping_rule""#));
}

#[test]
fn verify_bad_rla_statistical_exits_one_after_package_read() {
    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .args([
            "verify",
            &docs_bad_rla_statistical_path(),
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""equation_id":"rla_stopping_rule""#));
    assert!(stdout.contains("risk estimate mismatch"));
    assert!(stdout.contains(r#""equation_id":"source_hash_match","status":"pass""#));
}

#[test]
fn verify_colorado_rla_exposes_jurisdiction_adapter() {
    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .args(["verify", &docs_colorado_rla_path(), "--format", "json"])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""equation_id":"rla_jurisdiction_adapter""#));
    assert!(stdout.contains(r#""equation_id":"rla_stopping_rule""#));
}

#[test]
fn verify_bad_colorado_rla_exits_one_after_package_read() {
    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .args(["verify", &docs_bad_colorado_rla_path(), "--format", "json"])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""equation_id":"rla_jurisdiction_adapter""#));
    assert!(stdout.contains("invalid Colorado-style public seed"));
    assert!(stdout.contains(r#""equation_id":"source_hash_match","status":"pass""#));
}

#[test]
fn verify_california_rla_exposes_jurisdiction_adapter() {
    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .args(["verify", &docs_california_rla_path(), "--format", "json"])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""equation_id":"rla_jurisdiction_adapter""#));
    assert!(stdout.contains(r#""equation_id":"rla_stopping_rule""#));
}

#[test]
fn verify_bad_california_rla_exits_one_after_package_read() {
    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .args([
            "verify",
            &docs_bad_california_rla_path(),
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""equation_id":"rla_jurisdiction_adapter""#));
    assert!(stdout.contains("invalid public audit software source URL"));
    assert!(stdout.contains(r#""equation_id":"source_hash_match","status":"pass""#));
}

#[test]
fn verify_manual_audit_exposes_reconciliation() {
    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .args(["verify", &docs_manual_audit_path(), "--format", "json"])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""equation_id":"manual_audit_reconciliation""#));
}

#[test]
fn verify_bad_manual_audit_exits_one_after_package_read() {
    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .args(["verify", &docs_bad_manual_audit_path(), "--format", "json"])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""equation_id":"manual_audit_reconciliation""#));
    assert!(stdout.contains("computed Escalate"));
    assert!(stdout.contains(r#""equation_id":"source_hash_match","status":"pass""#));
}

#[test]
fn verify_tampered_manifest_exits_one() {
    let tmp = tempfile::TempDir::new().unwrap();
    copy_dir_all(std::path::Path::new(&docs_summary_basic_path()), tmp.path()).unwrap();
    let manifest_path = tmp.path().join("manifest.json");
    let mut raw: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&manifest_path).unwrap()).unwrap();
    raw["content_hash"] = serde_json::Value::String("sha256:bad".to_string());
    std::fs::write(&manifest_path, serde_json::to_vec_pretty(&raw).unwrap()).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .args(["verify", tmp.path().to_str().unwrap(), "--format", "json"])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""status":"fail""#));
    assert!(stdout.contains("content_hash mismatch"));
}

#[test]
fn verify_tampered_source_exits_one() {
    let tmp = tempfile::TempDir::new().unwrap();
    copy_dir_all(std::path::Path::new(&docs_summary_basic_path()), tmp.path()).unwrap();
    std::fs::write(
        tmp.path()
            .join("sources")
            .join("synthetic-summary-export.json"),
        br#"{"tampered":true}"#,
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .args(["verify", tmp.path().to_str().unwrap(), "--format", "json"])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""status":"fail""#));
    assert!(stdout.contains(r#""equation_id":"source_hash_match""#));
}

#[test]
fn verify_missing_source_hash_exits_one() {
    let tmp = tempfile::TempDir::new().unwrap();
    copy_dir_all(std::path::Path::new(&docs_summary_basic_path()), tmp.path()).unwrap();
    std::fs::write(
        tmp.path().join("sources").join("source-index.json"),
        br#"{"sources":[]}"#,
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .args(["verify", tmp.path().to_str().unwrap(), "--format", "json"])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""equation_id":"source_hash_match""#));
    assert!(stdout.contains("source index is empty"));
}

#[test]
fn verify_can_write_transcript_to_package() {
    let tmp = tempfile::TempDir::new().unwrap();
    copy_dir_all(std::path::Path::new(&docs_summary_basic_path()), tmp.path()).unwrap();
    let transcript_path = tmp
        .path()
        .join("transcripts")
        .join("verify-transcript.json");
    std::fs::remove_file(&transcript_path).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .args([
            "verify",
            tmp.path().to_str().unwrap(),
            "--write-transcript",
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let transcript = std::fs::read_to_string(transcript_path).unwrap();
    assert!(transcript.contains(r#""verifier": "rcount-audit""#));
}

#[test]
fn aggregate_districts_with_rplan_outputs_district_totals() {
    let dir = docs_district_aggregation_dir();
    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .args([
            "aggregate-districts",
            dir.join("package").to_str().unwrap(),
            "--plan",
            dir.join("plan.rplan.json").to_str().unwrap(),
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""equation_id":"district_aggregation_total""#));
    assert!(stdout.contains(r#""district_label":"SYN-D1""#));
    assert!(stdout.contains(r#""counted_ballots":80"#));
    assert!(stdout.contains(r#""rplan_plan_hash":"sha256:"#));
}

#[test]
fn aggregate_districts_outputs_declared_rctx_reference() {
    let tmp = tempfile::tempdir().unwrap();
    let package_dir = tmp.path().join("package");
    let plan_path = tmp.path().join("plan.rplan.json");
    let context_path = tmp.path().join("context.rctx");
    let crosswalk_path = tmp.path().join("crosswalks.ndjson");

    let mut package = rcount_core::synthetic_summary_basic_package();
    let mut units = rplan_core::PlanUnitIndex {
        unit_kind: rplan_core::UnitKind::Precinct,
        state: Some("SYN".to_string()),
        year: Some(2024),
        canonical_order: rplan_core::CanonicalOrder::ExplicitUnitIds,
        unit_ids: vec![
            "syn:precinct:P-001".to_string(),
            "syn:precinct:P-002".to_string(),
        ],
        unit_universe_hash: String::new(),
        source_id: Some("rcount:summary-basic".to_string()),
    };
    units.unit_universe_hash = units.compute_unit_universe_hash().unwrap();
    let plan = rplan_core::DistrictPlan {
        schema_version: rplan_core::DISTRICT_PLAN_SCHEMA_VERSION.to_string(),
        units: units.clone(),
        assignment: vec![0, 1],
        k: 2,
        display_labels: vec!["SYN-D1".to_string(), "SYN-D2".to_string()],
        allow_empty_districts: false,
    };
    plan.validate().unwrap();
    let mut context = rplan_core::RplanContext {
        rctx_version: rplan_core::RCTX_VERSION.to_string(),
        context_hash: String::new(),
        units,
        graph: None,
        populations: None,
        subdivisions: None,
        demographics: None,
        geometry: None,
        source_hashes: rplan_core::SourceHashes::default(),
    };
    context.context_hash = context.compute_context_hash().unwrap();
    let crosswalks = context
        .units
        .unit_ids
        .iter()
        .map(|unit_id| rctx_core::CrosswalkRecord {
            crosswalk_id: "cw-summary-basic-identity".to_string(),
            from_context_hash: context.context_hash.clone(),
            to_context_hash: context.context_hash.clone(),
            from_unit_id: unit_id.clone(),
            to_unit_id: unit_id.clone(),
            weight: rctx_core::RationalWeight { num: 1, den: 1 },
            weight_kind: rctx_core::CrosswalkWeightKind::UnitCount,
            exhaustive: true,
            source_refs: Vec::new(),
        })
        .collect::<Vec<_>>();
    let crosswalk_hash = rctx_core::crosswalk_set_hash(&crosswalks).unwrap();
    package.rctx_refs = vec![rcount_core::RctxReference {
        reference_id: "rctx:summary-basic-to-plan".to_string(),
        context_hash: context.context_hash.clone(),
        context_path: Some("context.rctx".to_string()),
        crosswalk_hash: Some(crosswalk_hash.clone()),
        crosswalk_path: Some("crosswalks/summary-basic-to-plan.ndjson".to_string()),
        role: "aggregation-crosswalk".to_string(),
        note: None,
    }];

    let manifest = rcount_io::synthetic_summary_basic_manifest(&package).unwrap();
    rcount_io::write_package_dir(&package_dir, &manifest, &package).unwrap();
    let plan_doc = rplan_io::RplanDocument {
        rplan_version: rplan_io::RPLAN_V02.to_string(),
        plan,
        metadata: rplan_io::RplanMetadataV02 {
            label: "synthetic-count-districts".to_string(),
            jurisdiction: "SYN".to_string(),
            chamber: "county-council".to_string(),
            created_at: "2026-05-12T00:00:00Z".to_string(),
            description: None,
        },
        provenance: rplan_io::RplanProvenance::default(),
        geometry: None,
        extensions: std::collections::BTreeMap::new(),
    };
    std::fs::write(&plan_path, rplan_io::write_rplan_string(&plan_doc).unwrap()).unwrap();
    std::fs::write(
        &context_path,
        rplan_io::write_rctx_string(&context).unwrap(),
    )
    .unwrap();
    let crosswalk_text = crosswalks
        .iter()
        .map(|record| serde_json::to_string(record).unwrap())
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(&crosswalk_path, format!("{crosswalk_text}\n")).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .args([
            "aggregate-districts",
            package_dir.to_str().unwrap(),
            "--plan",
            plan_path.to_str().unwrap(),
            "--context",
            context_path.to_str().unwrap(),
            "--crosswalk",
            crosswalk_path.to_str().unwrap(),
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""rctx_reference_id":"rctx:summary-basic-to-plan""#));
    assert!(stdout.contains(&format!(r#""rctx_crosswalk_hash":"{crosswalk_hash}""#)));
}

#[test]
fn verify_multi_election_cycle_uses_package_contest_for_jurisdiction_total() {
    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .args([
            "verify",
            &docs_multi_election_cycle_path("SYN-2028-general"),
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""status":"pass""#));
    assert!(stdout.contains(r#""contest_id":"syn-cycle-mayor""#));
    assert!(stdout.contains(r#""equation_id":"jurisdiction_contest_total""#));
}

#[test]
fn verify_bad_multi_election_lineage_exits_one() {
    let cycle_dir = docs_multi_election_negative_path("bad-lineage", "SYN-2028-general");
    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .args([
            "verify",
            cycle_dir.join("package").to_str().unwrap(),
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""equation_id":"lineage_conservation""#));
    assert!(stdout.contains("references missing current reporting unit"));
}

#[test]
fn aggregate_stale_multi_election_plan_exits_two() {
    let cycle_dir = docs_multi_election_negative_path("stale-plan", "SYN-2028-general");
    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .args([
            "aggregate-districts",
            cycle_dir.join("package").to_str().unwrap(),
            "--plan",
            cycle_dir.join("plan.rplan.json").to_str().unwrap(),
            "--contest-id",
            "syn-cycle-mayor",
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("missing plan unit summary"));
    assert!(stderr.contains("syn:precinct:P-002"));
}

#[test]
fn verify_tampered_multi_election_source_exits_one() {
    let cycle_dir = docs_multi_election_negative_path("tampered-2028-source", "SYN-2028-general");
    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .args([
            "verify",
            cycle_dir.join("package").to_str().unwrap(),
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""equation_id":"source_hash_match""#));
    assert!(stdout.contains("source hash mismatch"));
}

#[test]
fn replay_audit_algorithms_outputs_bravo_transcript() {
    let tmp = tempfile::tempdir().unwrap();
    let mut package = rcount_core::synthetic_summary_basic_package();
    package.audit_algorithm_runs = vec![AuditAlgorithmRun {
        run_id: "audit-run:bravo-toy".to_string(),
        contest_id: "syn-2024-mayor".to_string(),
        method_id: BRAVO_BALLOT_POLLING_METHOD_ID.to_string(),
        sampling_mode: AuditSamplingMode::WithReplacement,
        rcv_elimination_order: Vec::new(),
        risk_limit_ppm: Some(100_000),
        reported_winner_votes: Some(3),
        reported_loser_votes: Some(1),
        macro_ballot_count: None,
        macro_reported_margin: None,
        macro_gamma: None,
        combining_rule_id: None,
        nuisance_parameter: None,
        bayesian_prior_id: None,
        bayesian_likelihood_id: None,
        posterior_winner_probability_ppm: None,
        posterior_risk_ppm: None,
        simulation_seed: None,
        posterior_draws: None,
        calibrated_risk_limit_ppm: None,
        strata: Vec::new(),
        assertions: vec![AuditAssertion {
            assertion_id: "assertion:cand-a-over-cand-b".to_string(),
            kind: AuditAssertionKind::PluralityWinnerLoser,
            assorter_id: "plurality-winner-loser-v1".to_string(),
            assorter_upper_bound: RationalValue {
                numerator: 1,
                denominator: 1,
            },
            winner_selection_id: Some("cand-a".to_string()),
            loser_selection_id: Some("cand-b".to_string()),
        }],
        sample_steps: (0..6)
            .map(|step_index| AuditSampleStep {
                step_index,
                round_index: None,
                assertion_id: "assertion:cand-a-over-cand-b".to_string(),
                sample_unit_id: format!("ballot:{step_index}"),
                assorter_value: RationalValue {
                    numerator: 1,
                    denominator: 1,
                },
                bet: None,
                statistic: None,
                p_value_ppm: None,
                ranked_choices: Vec::new(),
                source_refs: Vec::new(),
            })
            .collect(),
        decision: AuditAlgorithmDecision::Pass,
        source_refs: Vec::new(),
    }];
    let manifest = synthetic_summary_basic_manifest(&package).unwrap();
    write_package_dir(tmp.path(), &manifest, &package).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .args([
            "replay-audit-algorithms",
            tmp.path().to_str().unwrap(),
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""run_id":"audit-run:bravo-toy""#));
    assert!(stdout.contains(r#""status":"pass""#));
    assert!(stdout.contains(r#""computed_decision":"pass""#));
    assert!(stdout.contains(r#""p_value_ppm":87792"#));
}

#[test]
fn replay_audit_algorithms_outputs_minerva_round_one_transcript() {
    let tmp = tempfile::tempdir().unwrap();
    let package = rcount_core::synthetic_minerva_round_one_package();
    let manifest = synthetic_summary_basic_manifest(&package).unwrap();
    write_package_dir(tmp.path(), &manifest, &package).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .args([
            "replay-audit-algorithms",
            tmp.path().to_str().unwrap(),
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""run_id":"audit-run:minerva-round-one-pass""#));
    assert!(stdout.contains(r#""method_id":"minerva-ballot-polling-v1""#));
    assert!(stdout.contains(r#""status":"pass""#));
    assert!(stdout.contains(r#""computed_decision":"pass""#));
    assert!(stdout.contains(r#""statistic":{"numerator":729,"denominator":64}"#));
    assert!(stdout.contains(r#""p_value_ppm":87792"#));
}

#[test]
fn replay_audit_algorithms_outputs_minerva_multi_round_transcript() {
    let tmp = tempfile::tempdir().unwrap();
    let package = rcount_core::synthetic_minerva_multi_round_package();
    let manifest = synthetic_summary_basic_manifest(&package).unwrap();
    write_package_dir(tmp.path(), &manifest, &package).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .args([
            "replay-audit-algorithms",
            tmp.path().to_str().unwrap(),
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""run_id":"audit-run:minerva-multi-round-pass""#));
    assert!(stdout.contains(r#""status":"pass""#));
    assert!(stdout.contains(r#""step_index":4"#));
    assert!(stdout.contains(r#""p_value_ppm":131688"#));
    assert!(stdout.contains(r#""stop":false"#));
    assert!(stdout.contains(r#""step_index":5"#));
    assert!(stdout.contains(r#""p_value_ppm":87792"#));
    assert!(stdout.contains(r#""stop":true"#));
}

#[test]
fn replay_audit_algorithms_exits_one_on_minerva_declared_drift() {
    let tmp = tempfile::tempdir().unwrap();
    let mut package = rcount_core::synthetic_minerva_round_one_package();
    package.audit_algorithm_runs[0].sample_steps[5].p_value_ppm = Some(999_999);
    let manifest = synthetic_summary_basic_manifest(&package).unwrap();
    write_package_dir(tmp.path(), &manifest, &package).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .args([
            "replay-audit-algorithms",
            tmp.path().to_str().unwrap(),
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""status":"fail""#));
    assert!(stdout.contains("declared p-value mismatch"));
}

#[test]
fn replay_audit_algorithms_outputs_athena_boundary() {
    let tmp = tempfile::tempdir().unwrap();
    let package = rcount_core::synthetic_athena_boundary_package();
    let manifest = synthetic_summary_basic_manifest(&package).unwrap();
    write_package_dir(tmp.path(), &manifest, &package).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .args([
            "replay-audit-algorithms",
            tmp.path().to_str().unwrap(),
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""run_id":"audit-run:athena-boundary""#));
    assert!(stdout.contains(r#""method_id":"athena-ballot-polling-v1""#));
    assert!(stdout.contains(r#""status":"boundary""#));
    assert!(stdout.contains("Athena round risk calculation is recorded but not replayed"));
}

#[test]
fn replay_audit_algorithms_outputs_stratified_hybrid_boundary() {
    let tmp = tempfile::tempdir().unwrap();
    let package = rcount_core::synthetic_stratified_hybrid_package();
    let manifest = synthetic_summary_basic_manifest(&package).unwrap();
    write_package_dir(tmp.path(), &manifest, &package).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .args([
            "replay-audit-algorithms",
            tmp.path().to_str().unwrap(),
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""run_id":"audit-run:stratified-hybrid-boundary""#));
    assert!(stdout.contains(r#""method_id":"stratified-hybrid-rla-v1""#));
    assert!(stdout.contains(r#""status":"boundary""#));
    assert!(stdout.contains("stratified/hybrid combined-risk replay is recorded but not replayed"));
}

#[test]
fn replay_audit_algorithms_outputs_raire_boundary() {
    let tmp = tempfile::tempdir().unwrap();
    let package = rcount_core::synthetic_raire_boundary_package();
    let manifest = synthetic_summary_basic_manifest(&package).unwrap();
    write_package_dir(tmp.path(), &manifest, &package).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .args([
            "replay-audit-algorithms",
            tmp.path().to_str().unwrap(),
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""run_id":"audit-run:raire-irv-boundary""#));
    assert!(stdout.contains(r#""method_id":"raire-irv-v1""#));
    assert!(stdout.contains(r#""status":"boundary""#));
    assert!(stdout.contains("RAIRE IRV assertion replay is recorded but not replayed"));
}

#[test]
fn replay_audit_algorithms_outputs_awaire_boundary() {
    let tmp = tempfile::tempdir().unwrap();
    let package = rcount_core::synthetic_awaire_boundary_package();
    let manifest = synthetic_summary_basic_manifest(&package).unwrap();
    write_package_dir(tmp.path(), &manifest, &package).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .args([
            "replay-audit-algorithms",
            tmp.path().to_str().unwrap(),
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""run_id":"audit-run:awaire-irv-boundary""#));
    assert!(stdout.contains(r#""method_id":"awaire-irv-v1""#));
    assert!(stdout.contains(r#""status":"boundary""#));
    assert!(stdout.contains("AWAIRE IRV adaptive replay is recorded but not replayed"));
}

#[test]
fn replay_audit_algorithms_outputs_bayesian_boundary() {
    let tmp = tempfile::tempdir().unwrap();
    let package = rcount_core::synthetic_bayesian_tabulation_boundary_package();
    let manifest = synthetic_summary_basic_manifest(&package).unwrap();
    write_package_dir(tmp.path(), &manifest, &package).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .args([
            "replay-audit-algorithms",
            tmp.path().to_str().unwrap(),
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""run_id":"audit-run:bayesian-tabulation-boundary""#));
    assert!(stdout.contains(r#""method_id":"bayesian-tabulation-audit-v1""#));
    assert!(stdout.contains(r#""status":"boundary""#));
    assert!(stdout.contains("Bayesian tabulation posterior analytics"));
}

#[test]
fn replay_audit_algorithms_outputs_soba_boundary() {
    let tmp = tempfile::tempdir().unwrap();
    let package = rcount_core::synthetic_soba_observable_ballot_boundary_package();
    let manifest = synthetic_summary_basic_manifest(&package).unwrap();
    write_package_dir(tmp.path(), &manifest, &package).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .args([
            "replay-audit-algorithms",
            tmp.path().to_str().unwrap(),
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""run_id":"audit-run:soba-observable-ballot-boundary""#));
    assert!(stdout.contains(r#""method_id":"soba-observable-ballot-audit-v1""#));
    assert!(stdout.contains(r#""status":"boundary""#));
    assert!(stdout.contains("SOBA observable-ballot linkage"));
}

#[test]
fn replay_audit_algorithms_exits_one_on_declared_statistic_drift() {
    let tmp = tempfile::tempdir().unwrap();
    let mut package = rcount_core::synthetic_summary_basic_package();
    let mut steps = (0..6)
        .map(|step_index| AuditSampleStep {
            step_index,
            round_index: None,
            assertion_id: "assertion:cand-a-over-cand-b".to_string(),
            sample_unit_id: format!("ballot:{step_index}"),
            assorter_value: RationalValue {
                numerator: 1,
                denominator: 1,
            },
            bet: None,
            statistic: None,
            p_value_ppm: None,
            ranked_choices: Vec::new(),
            source_refs: Vec::new(),
        })
        .collect::<Vec<_>>();
    steps[5].p_value_ppm = Some(999_999);
    package.audit_algorithm_runs = vec![AuditAlgorithmRun {
        run_id: "audit-run:bravo-drift".to_string(),
        contest_id: "syn-2024-mayor".to_string(),
        method_id: BRAVO_BALLOT_POLLING_METHOD_ID.to_string(),
        sampling_mode: AuditSamplingMode::WithReplacement,
        rcv_elimination_order: Vec::new(),
        risk_limit_ppm: Some(100_000),
        reported_winner_votes: Some(3),
        reported_loser_votes: Some(1),
        macro_ballot_count: None,
        macro_reported_margin: None,
        macro_gamma: None,
        combining_rule_id: None,
        nuisance_parameter: None,
        bayesian_prior_id: None,
        bayesian_likelihood_id: None,
        posterior_winner_probability_ppm: None,
        posterior_risk_ppm: None,
        simulation_seed: None,
        posterior_draws: None,
        calibrated_risk_limit_ppm: None,
        strata: Vec::new(),
        assertions: vec![AuditAssertion {
            assertion_id: "assertion:cand-a-over-cand-b".to_string(),
            kind: AuditAssertionKind::PluralityWinnerLoser,
            assorter_id: "plurality-winner-loser-v1".to_string(),
            assorter_upper_bound: RationalValue {
                numerator: 1,
                denominator: 1,
            },
            winner_selection_id: Some("cand-a".to_string()),
            loser_selection_id: Some("cand-b".to_string()),
        }],
        sample_steps: steps,
        decision: AuditAlgorithmDecision::Pass,
        source_refs: Vec::new(),
    }];
    let manifest = synthetic_summary_basic_manifest(&package).unwrap();
    write_package_dir(tmp.path(), &manifest, &package).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .args([
            "replay-audit-algorithms",
            tmp.path().to_str().unwrap(),
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""status":"fail""#));
    assert!(stdout.contains("declared p-value mismatch"));
}

#[test]
fn replay_audit_algorithms_outputs_boundary_for_missing_bravo_votes() {
    let tmp = tempfile::tempdir().unwrap();
    let mut package = rcount_core::synthetic_summary_basic_package();
    package.audit_algorithm_runs = vec![AuditAlgorithmRun {
        run_id: "audit-run:bravo-boundary".to_string(),
        contest_id: "syn-2024-mayor".to_string(),
        method_id: BRAVO_BALLOT_POLLING_METHOD_ID.to_string(),
        sampling_mode: AuditSamplingMode::WithReplacement,
        rcv_elimination_order: Vec::new(),
        risk_limit_ppm: Some(100_000),
        reported_winner_votes: None,
        reported_loser_votes: Some(1),
        macro_ballot_count: None,
        macro_reported_margin: None,
        macro_gamma: None,
        combining_rule_id: None,
        nuisance_parameter: None,
        bayesian_prior_id: None,
        bayesian_likelihood_id: None,
        posterior_winner_probability_ppm: None,
        posterior_risk_ppm: None,
        simulation_seed: None,
        posterior_draws: None,
        calibrated_risk_limit_ppm: None,
        strata: Vec::new(),
        assertions: vec![AuditAssertion {
            assertion_id: "assertion:cand-a-over-cand-b".to_string(),
            kind: AuditAssertionKind::PluralityWinnerLoser,
            assorter_id: "plurality-winner-loser-v1".to_string(),
            assorter_upper_bound: RationalValue {
                numerator: 1,
                denominator: 1,
            },
            winner_selection_id: Some("cand-a".to_string()),
            loser_selection_id: Some("cand-b".to_string()),
        }],
        sample_steps: vec![AuditSampleStep {
            step_index: 0,
            round_index: None,
            assertion_id: "assertion:cand-a-over-cand-b".to_string(),
            sample_unit_id: "ballot:0".to_string(),
            assorter_value: RationalValue {
                numerator: 1,
                denominator: 1,
            },
            bet: None,
            statistic: None,
            p_value_ppm: None,
            ranked_choices: Vec::new(),
            source_refs: Vec::new(),
        }],
        decision: AuditAlgorithmDecision::Boundary,
        source_refs: Vec::new(),
    }];
    let manifest = synthetic_summary_basic_manifest(&package).unwrap();
    write_package_dir(tmp.path(), &manifest, &package).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .args([
            "replay-audit-algorithms",
            tmp.path().to_str().unwrap(),
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""status":"boundary""#));
    assert!(stdout.contains("BRAVO replay requires reported_winner_votes"));
}

#[test]
fn replay_audit_algorithms_outputs_continue_for_nonstopping_bravo_run() {
    let tmp = tempfile::tempdir().unwrap();
    let mut package = rcount_core::synthetic_summary_basic_package();
    package.audit_algorithm_runs = vec![AuditAlgorithmRun {
        run_id: "audit-run:bravo-continue".to_string(),
        contest_id: "syn-2024-mayor".to_string(),
        method_id: BRAVO_BALLOT_POLLING_METHOD_ID.to_string(),
        sampling_mode: AuditSamplingMode::WithReplacement,
        rcv_elimination_order: Vec::new(),
        risk_limit_ppm: Some(100_000),
        reported_winner_votes: Some(3),
        reported_loser_votes: Some(1),
        macro_ballot_count: None,
        macro_reported_margin: None,
        macro_gamma: None,
        combining_rule_id: None,
        nuisance_parameter: None,
        bayesian_prior_id: None,
        bayesian_likelihood_id: None,
        posterior_winner_probability_ppm: None,
        posterior_risk_ppm: None,
        simulation_seed: None,
        posterior_draws: None,
        calibrated_risk_limit_ppm: None,
        strata: Vec::new(),
        assertions: vec![AuditAssertion {
            assertion_id: "assertion:cand-a-over-cand-b".to_string(),
            kind: AuditAssertionKind::PluralityWinnerLoser,
            assorter_id: "plurality-winner-loser-v1".to_string(),
            assorter_upper_bound: RationalValue {
                numerator: 1,
                denominator: 1,
            },
            winner_selection_id: Some("cand-a".to_string()),
            loser_selection_id: Some("cand-b".to_string()),
        }],
        sample_steps: vec![AuditSampleStep {
            step_index: 0,
            round_index: None,
            assertion_id: "assertion:cand-a-over-cand-b".to_string(),
            sample_unit_id: "ballot:0".to_string(),
            assorter_value: RationalValue {
                numerator: 1,
                denominator: 1,
            },
            bet: None,
            statistic: None,
            p_value_ppm: None,
            ranked_choices: Vec::new(),
            source_refs: Vec::new(),
        }],
        decision: AuditAlgorithmDecision::Continue,
        source_refs: Vec::new(),
    }];
    let manifest = synthetic_summary_basic_manifest(&package).unwrap();
    write_package_dir(tmp.path(), &manifest, &package).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .args([
            "replay-audit-algorithms",
            tmp.path().to_str().unwrap(),
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""status":"pass""#));
    assert!(stdout.contains(r#""computed_decision":"continue""#));
    assert!(stdout.contains(r#""statistic":{"numerator":3,"denominator":2}"#));
}

#[test]
fn replay_audit_algorithms_outputs_alpha_transcript() {
    let tmp = tempfile::tempdir().unwrap();
    let mut package = rcount_core::synthetic_summary_basic_package();
    package.audit_algorithm_runs = vec![alpha_toy_run(
        "audit-run:alpha-toy",
        AuditAlgorithmDecision::Pass,
        Some(RationalValue {
            numerator: 1,
            denominator: 1,
        }),
        None,
    )];
    let manifest = synthetic_summary_basic_manifest(&package).unwrap();
    write_package_dir(tmp.path(), &manifest, &package).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .args([
            "replay-audit-algorithms",
            tmp.path().to_str().unwrap(),
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""run_id":"audit-run:alpha-toy""#));
    assert!(stdout.contains(r#""status":"pass""#));
    assert!(stdout.contains(r#""computed_decision":"pass""#));
    assert!(stdout.contains(r#""statistic":{"numerator":81,"denominator":16}"#));
    assert!(stdout.contains(r#""p_value_ppm":197531"#));
}

#[test]
fn replay_audit_algorithms_exits_one_on_alpha_declared_drift() {
    let tmp = tempfile::tempdir().unwrap();
    let mut package = rcount_core::synthetic_summary_basic_package();
    package.audit_algorithm_runs = vec![alpha_toy_run(
        "audit-run:alpha-drift",
        AuditAlgorithmDecision::Pass,
        Some(RationalValue {
            numerator: 1,
            denominator: 1,
        }),
        Some(999_999),
    )];
    let manifest = synthetic_summary_basic_manifest(&package).unwrap();
    write_package_dir(tmp.path(), &manifest, &package).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .args([
            "replay-audit-algorithms",
            tmp.path().to_str().unwrap(),
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""status":"fail""#));
    assert!(stdout.contains("declared p-value mismatch"));
}

#[test]
fn replay_audit_algorithms_outputs_alpha_boundary_when_bets_are_missing() {
    let tmp = tempfile::tempdir().unwrap();
    let mut package = rcount_core::synthetic_summary_basic_package();
    package.audit_algorithm_runs = vec![alpha_toy_run(
        "audit-run:alpha-boundary",
        AuditAlgorithmDecision::Boundary,
        None,
        None,
    )];
    let manifest = synthetic_summary_basic_manifest(&package).unwrap();
    write_package_dir(tmp.path(), &manifest, &package).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .args([
            "replay-audit-algorithms",
            tmp.path().to_str().unwrap(),
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""status":"boundary""#));
    assert!(stdout.contains("ALPHA replay requires bet on every sample step"));
}

#[test]
fn replay_audit_algorithms_outputs_kaplan_markov_taint_product_continue() {
    let tmp = tempfile::tempdir().unwrap();
    let mut package = rcount_core::synthetic_summary_basic_package();
    package.audit_algorithm_runs = vec![AuditAlgorithmRun {
        run_id: "audit-run:kaplan-markov-continue".to_string(),
        contest_id: "syn-2024-mayor".to_string(),
        method_id: KAPLAN_MARKOV_COMPARISON_METHOD_ID.to_string(),
        sampling_mode: AuditSamplingMode::WithoutReplacement,
        rcv_elimination_order: Vec::new(),
        risk_limit_ppm: Some(100_000),
        reported_winner_votes: Some(62),
        reported_loser_votes: Some(35),
        macro_ballot_count: None,
        macro_reported_margin: None,
        macro_gamma: None,
        combining_rule_id: None,
        nuisance_parameter: None,
        bayesian_prior_id: None,
        bayesian_likelihood_id: None,
        posterior_winner_probability_ppm: None,
        posterior_risk_ppm: None,
        simulation_seed: None,
        posterior_draws: None,
        calibrated_risk_limit_ppm: None,
        strata: Vec::new(),
        assertions: vec![AuditAssertion {
            assertion_id: "assertion:cand-a-over-cand-b".to_string(),
            kind: AuditAssertionKind::ComparisonOverstatement,
            assorter_id: "plurality-comparison-overstatement-v1".to_string(),
            assorter_upper_bound: RationalValue {
                numerator: 2,
                denominator: 1,
            },
            winner_selection_id: Some("cand-a".to_string()),
            loser_selection_id: Some("cand-b".to_string()),
        }],
        sample_steps: vec![AuditSampleStep {
            step_index: 0,
            round_index: None,
            assertion_id: "assertion:cand-a-over-cand-b".to_string(),
            sample_unit_id: "ballot:comparison:0".to_string(),
            assorter_value: RationalValue {
                numerator: 0,
                denominator: 1,
            },
            bet: None,
            statistic: None,
            p_value_ppm: None,
            ranked_choices: Vec::new(),
            source_refs: vec![
                "cvr:ballot:comparison:0".to_string(),
                "hand-interpretation:ballot:comparison:0".to_string(),
            ],
        }],
        decision: AuditAlgorithmDecision::Continue,
        source_refs: vec!["source:synthetic-comparison-audit".to_string()],
    }];
    let manifest = synthetic_summary_basic_manifest(&package).unwrap();
    write_package_dir(tmp.path(), &manifest, &package).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .args([
            "replay-audit-algorithms",
            tmp.path().to_str().unwrap(),
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""method_id":"kaplan-markov-comparison-v1""#));
    assert!(stdout.contains(r#""status":"pass""#));
    assert!(stdout.contains(r#""computed_decision":"continue""#));
    assert!(stdout.contains(r#""p_value_ppm":1000000"#));
}

#[test]
fn replay_audit_algorithms_outputs_kaplan_markov_taint_product_pass() {
    let tmp = tempfile::tempdir().unwrap();
    let mut package = rcount_core::synthetic_summary_basic_package();
    package.audit_algorithm_runs = vec![comparison_taint_product_run(
        KAPLAN_MARKOV_COMPARISON_METHOD_ID,
        "audit-run:kaplan-markov-pass",
        AuditSamplingMode::WithoutReplacement,
        300_000,
        &[(1, 2), (1, 2)],
        AuditAlgorithmDecision::Pass,
        None,
    )];
    let manifest = synthetic_summary_basic_manifest(&package).unwrap();
    write_package_dir(tmp.path(), &manifest, &package).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .args([
            "replay-audit-algorithms",
            tmp.path().to_str().unwrap(),
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""method_id":"kaplan-markov-comparison-v1""#));
    assert!(stdout.contains(r#""computed_decision":"pass""#));
    assert!(stdout.contains(r#""p_value_ppm":250000"#));
    assert!(stdout.contains(r#""stop":true"#));
}

#[test]
fn replay_audit_algorithms_outputs_kaplan_markov_macro_pass() {
    let tmp = tempfile::tempdir().unwrap();
    let package = rcount_core::synthetic_kaplan_markov_macro_package();
    let manifest = synthetic_summary_basic_manifest(&package).unwrap();
    write_package_dir(tmp.path(), &manifest, &package).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .args([
            "replay-audit-algorithms",
            tmp.path().to_str().unwrap(),
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""method_id":"kaplan-markov-comparison-v1""#));
    assert!(stdout.contains(r#""computed_decision":"pass""#));
    assert!(stdout.contains(r#""p_value_ppm":475058"#));
    assert!(stdout.contains(r#""stop":true"#));
}

#[test]
fn verify_kaplan_markov_macro_partial_design_exits_one() {
    let tmp = tempfile::tempdir().unwrap();
    let mut package = rcount_core::synthetic_summary_basic_package();
    let mut run = comparison_taint_product_run(
        KAPLAN_MARKOV_COMPARISON_METHOD_ID,
        "audit-run:kaplan-markov-macro-partial",
        AuditSamplingMode::WithoutReplacement,
        500_000,
        &[(0, 1)],
        AuditAlgorithmDecision::Continue,
        None,
    );
    run.macro_ballot_count = Some(100);
    package.audit_algorithm_runs = vec![run];
    let manifest = synthetic_summary_basic_manifest(&package).unwrap();
    write_package_dir(tmp.path(), &manifest, &package).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .args(["verify", tmp.path().to_str().unwrap(), "--format", "json"])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""equation_id":"audit_algorithm_transcript""#));
    assert!(stdout.contains("invalid MACRO design fields"));
}

#[test]
fn replay_audit_algorithms_exits_one_on_kaplan_markov_declared_drift() {
    let tmp = tempfile::tempdir().unwrap();
    let mut package = rcount_core::synthetic_summary_basic_package();
    package.audit_algorithm_runs = vec![comparison_taint_product_run(
        KAPLAN_MARKOV_COMPARISON_METHOD_ID,
        "audit-run:kaplan-markov-drift",
        AuditSamplingMode::WithoutReplacement,
        300_000,
        &[(1, 2), (1, 2)],
        AuditAlgorithmDecision::Pass,
        Some(999_999),
    )];
    let manifest = synthetic_summary_basic_manifest(&package).unwrap();
    write_package_dir(tmp.path(), &manifest, &package).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .args([
            "replay-audit-algorithms",
            tmp.path().to_str().unwrap(),
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""status":"fail""#));
    assert!(stdout.contains("declared p-value mismatch"));
}

#[test]
fn replay_audit_algorithms_outputs_kaplan_markov_boundary_without_risk_limit() {
    let tmp = tempfile::tempdir().unwrap();
    let mut package = rcount_core::synthetic_summary_basic_package();
    package.audit_algorithm_runs = vec![AuditAlgorithmRun {
        run_id: "audit-run:kaplan-markov-boundary".to_string(),
        contest_id: "syn-2024-mayor".to_string(),
        method_id: KAPLAN_MARKOV_COMPARISON_METHOD_ID.to_string(),
        sampling_mode: AuditSamplingMode::WithoutReplacement,
        rcv_elimination_order: Vec::new(),
        risk_limit_ppm: None,
        reported_winner_votes: Some(62),
        reported_loser_votes: Some(35),
        macro_ballot_count: None,
        macro_reported_margin: None,
        macro_gamma: None,
        combining_rule_id: None,
        nuisance_parameter: None,
        bayesian_prior_id: None,
        bayesian_likelihood_id: None,
        posterior_winner_probability_ppm: None,
        posterior_risk_ppm: None,
        simulation_seed: None,
        posterior_draws: None,
        calibrated_risk_limit_ppm: None,
        strata: Vec::new(),
        assertions: vec![AuditAssertion {
            assertion_id: "assertion:cand-a-over-cand-b".to_string(),
            kind: AuditAssertionKind::ComparisonOverstatement,
            assorter_id: "plurality-comparison-overstatement-v1".to_string(),
            assorter_upper_bound: RationalValue {
                numerator: 2,
                denominator: 1,
            },
            winner_selection_id: Some("cand-a".to_string()),
            loser_selection_id: Some("cand-b".to_string()),
        }],
        sample_steps: vec![AuditSampleStep {
            step_index: 0,
            round_index: None,
            assertion_id: "assertion:cand-a-over-cand-b".to_string(),
            sample_unit_id: "ballot:comparison:0".to_string(),
            assorter_value: RationalValue {
                numerator: 0,
                denominator: 1,
            },
            bet: None,
            statistic: None,
            p_value_ppm: None,
            ranked_choices: Vec::new(),
            source_refs: vec![
                "cvr:ballot:comparison:0".to_string(),
                "hand-interpretation:ballot:comparison:0".to_string(),
            ],
        }],
        decision: AuditAlgorithmDecision::Boundary,
        source_refs: vec!["source:synthetic-comparison-audit".to_string()],
    }];
    let manifest = synthetic_summary_basic_manifest(&package).unwrap();
    write_package_dir(tmp.path(), &manifest, &package).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .args([
            "replay-audit-algorithms",
            tmp.path().to_str().unwrap(),
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""status":"boundary""#));
    assert!(stdout.contains("Kaplan-Markov taint-product replay requires risk_limit_ppm"));
}

#[test]
fn replay_audit_algorithms_outputs_batch_comparison_taint_product_pass() {
    let tmp = tempfile::tempdir().unwrap();
    let mut package = rcount_core::synthetic_summary_basic_package();
    package.audit_algorithm_runs = vec![comparison_taint_product_run(
        BATCH_COMPARISON_METHOD_ID,
        "audit-run:batch-comparison-pass",
        AuditSamplingMode::Batch,
        300_000,
        &[(1, 2), (1, 2)],
        AuditAlgorithmDecision::Pass,
        None,
    )];
    let manifest = synthetic_summary_basic_manifest(&package).unwrap();
    write_package_dir(tmp.path(), &manifest, &package).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .args([
            "replay-audit-algorithms",
            tmp.path().to_str().unwrap(),
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""method_id":"batch-comparison-v1""#));
    assert!(stdout.contains(r#""computed_decision":"pass""#));
    assert!(stdout.contains(r#""p_value_ppm":250000"#));
}

#[test]
fn replay_audit_algorithms_outputs_derived_batch_comparison_continue() {
    let tmp = tempfile::tempdir().unwrap();
    let package = rcount_core::synthetic_batch_comparison_algorithm_package();
    let manifest = synthetic_summary_basic_manifest(&package).unwrap();
    write_package_dir(tmp.path(), &manifest, &package).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .args([
            "replay-audit-algorithms",
            tmp.path().to_str().unwrap(),
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""method_id":"batch-comparison-v1""#));
    assert!(stdout.contains(r#""sample_unit_id":"batch:P-001:election-day""#));
    assert!(stdout.contains(r#""computed_decision":"continue""#));
    assert!(stdout.contains(r#""p_value_ppm":600000"#));
}

#[test]
fn replay_audit_algorithms_outputs_batch_comparison_boundary_without_risk_limit() {
    let tmp = tempfile::tempdir().unwrap();
    let mut package = rcount_core::synthetic_summary_basic_package();
    let mut run = comparison_taint_product_run(
        BATCH_COMPARISON_METHOD_ID,
        "audit-run:batch-comparison-boundary",
        AuditSamplingMode::Batch,
        300_000,
        &[(1, 2)],
        AuditAlgorithmDecision::Boundary,
        None,
    );
    run.risk_limit_ppm = None;
    package.audit_algorithm_runs = vec![run];
    let manifest = synthetic_summary_basic_manifest(&package).unwrap();
    write_package_dir(tmp.path(), &manifest, &package).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .args([
            "replay-audit-algorithms",
            tmp.path().to_str().unwrap(),
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""status":"boundary""#));
    assert!(stdout.contains("batch-comparison taint-product replay requires risk_limit_ppm"));
}

#[test]
fn verify_batch_comparison_package_exposes_overstatement_check() {
    let tmp = tempfile::tempdir().unwrap();
    let package = rcount_core::synthetic_batch_comparison_package();
    let manifest = synthetic_summary_basic_manifest(&package).unwrap();
    write_package_dir(tmp.path(), &manifest, &package).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .args(["verify", tmp.path().to_str().unwrap(), "--format", "json"])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""equation_id":"batch_comparison_overstatement""#));
    assert!(stdout.contains(r#""reporting_unit_id":"batch:P-001:election-day""#));
}

#[test]
fn verify_batch_comparison_algorithm_package_exposes_linkage_check() {
    let tmp = tempfile::tempdir().unwrap();
    let package = rcount_core::synthetic_batch_comparison_algorithm_package();
    let manifest = synthetic_summary_basic_manifest(&package).unwrap();
    write_package_dir(tmp.path(), &manifest, &package).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .args(["verify", tmp.path().to_str().unwrap(), "--format", "json"])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""equation_id":"batch_comparison_overstatement""#));
    assert!(stdout.contains(r#""equation_id":"batch_comparison_algorithm_linkage""#));
}

#[test]
fn verify_bad_batch_comparison_algorithm_exits_one() {
    let tmp = tempfile::tempdir().unwrap();
    let package = rcount_core::synthetic_bad_batch_comparison_algorithm_package();
    let manifest = synthetic_summary_basic_manifest(&package).unwrap();
    write_package_dir(tmp.path(), &manifest, &package).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .args(["verify", tmp.path().to_str().unwrap(), "--format", "json"])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""equation_id":"batch_comparison_algorithm_linkage""#));
    assert!(stdout.contains("batch comparison taint mismatch"));
}

#[test]
fn verify_missing_hand_tally_batch_comparison_exits_one() {
    let tmp = tempfile::tempdir().unwrap();
    let package = rcount_core::synthetic_missing_hand_tally_batch_comparison_package();
    let manifest = synthetic_summary_basic_manifest(&package).unwrap();
    write_package_dir(tmp.path(), &manifest, &package).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .args(["verify", tmp.path().to_str().unwrap(), "--format", "json"])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""equation_id":"batch_comparison_overstatement""#));
    assert!(stdout.contains("missing hand tally"));
}

#[test]
fn verify_batch_size_drift_comparison_exits_one() {
    let tmp = tempfile::tempdir().unwrap();
    let package = rcount_core::synthetic_batch_size_drift_comparison_package();
    let manifest = synthetic_summary_basic_manifest(&package).unwrap();
    write_package_dir(tmp.path(), &manifest, &package).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_rcount"))
        .args(["verify", tmp.path().to_str().unwrap(), "--format", "json"])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""equation_id":"batch_comparison_overstatement""#));
    assert!(stdout.contains("batch size mismatch"));
}

fn comparison_taint_product_run(
    method_id: &str,
    run_id: &str,
    sampling_mode: AuditSamplingMode,
    risk_limit_ppm: u32,
    taints: &[(i64, i64)],
    decision: AuditAlgorithmDecision,
    declared_last_p_value_ppm: Option<u32>,
) -> AuditAlgorithmRun {
    let mut sample_steps = taints
        .iter()
        .enumerate()
        .map(|(step_index, (numerator, denominator))| AuditSampleStep {
            step_index: step_index as u32,
            round_index: None,
            assertion_id: "assertion:cand-a-over-cand-b".to_string(),
            sample_unit_id: format!("comparison-unit:{step_index}"),
            assorter_value: RationalValue {
                numerator: *numerator,
                denominator: *denominator,
            },
            bet: None,
            statistic: None,
            p_value_ppm: None,
            ranked_choices: Vec::new(),
            source_refs: Vec::new(),
        })
        .collect::<Vec<_>>();
    if let Some(declared) = declared_last_p_value_ppm {
        let last = sample_steps
            .last_mut()
            .expect("comparison fixture must include at least one sample step");
        last.p_value_ppm = Some(declared);
    }

    AuditAlgorithmRun {
        run_id: run_id.to_string(),
        contest_id: "syn-2024-mayor".to_string(),
        method_id: method_id.to_string(),
        sampling_mode,
        rcv_elimination_order: Vec::new(),
        risk_limit_ppm: Some(risk_limit_ppm),
        reported_winner_votes: Some(62),
        reported_loser_votes: Some(35),
        macro_ballot_count: None,
        macro_reported_margin: None,
        macro_gamma: None,
        combining_rule_id: None,
        nuisance_parameter: None,
        bayesian_prior_id: None,
        bayesian_likelihood_id: None,
        posterior_winner_probability_ppm: None,
        posterior_risk_ppm: None,
        simulation_seed: None,
        posterior_draws: None,
        calibrated_risk_limit_ppm: None,
        strata: Vec::new(),
        assertions: vec![AuditAssertion {
            assertion_id: "assertion:cand-a-over-cand-b".to_string(),
            kind: AuditAssertionKind::ComparisonOverstatement,
            assorter_id: "plurality-comparison-taint-v1".to_string(),
            assorter_upper_bound: RationalValue {
                numerator: 1,
                denominator: 1,
            },
            winner_selection_id: Some("cand-a".to_string()),
            loser_selection_id: Some("cand-b".to_string()),
        }],
        sample_steps,
        decision,
        source_refs: Vec::new(),
    }
}

fn alpha_toy_run(
    run_id: &str,
    decision: AuditAlgorithmDecision,
    bet: Option<RationalValue>,
    declared_last_p_value_ppm: Option<u32>,
) -> AuditAlgorithmRun {
    let mut sample_steps = (0..4)
        .map(|step_index| AuditSampleStep {
            step_index,
            round_index: None,
            assertion_id: "assertion:cand-a-over-cand-b".to_string(),
            sample_unit_id: format!("ballot:{step_index}"),
            assorter_value: RationalValue {
                numerator: 1,
                denominator: 1,
            },
            bet,
            statistic: None,
            p_value_ppm: None,
            ranked_choices: Vec::new(),
            source_refs: Vec::new(),
        })
        .collect::<Vec<_>>();
    sample_steps[3].p_value_ppm = declared_last_p_value_ppm;

    AuditAlgorithmRun {
        run_id: run_id.to_string(),
        contest_id: "syn-2024-mayor".to_string(),
        method_id: ALPHA_MARTINGALE_METHOD_ID.to_string(),
        sampling_mode: AuditSamplingMode::WithReplacement,
        rcv_elimination_order: Vec::new(),
        risk_limit_ppm: Some(250_000),
        reported_winner_votes: None,
        reported_loser_votes: None,
        macro_ballot_count: None,
        macro_reported_margin: None,
        macro_gamma: None,
        combining_rule_id: None,
        nuisance_parameter: None,
        bayesian_prior_id: None,
        bayesian_likelihood_id: None,
        posterior_winner_probability_ppm: None,
        posterior_risk_ppm: None,
        simulation_seed: None,
        posterior_draws: None,
        calibrated_risk_limit_ppm: None,
        strata: Vec::new(),
        assertions: vec![AuditAssertion {
            assertion_id: "assertion:cand-a-over-cand-b".to_string(),
            kind: AuditAssertionKind::AssorterMean,
            assorter_id: "toy-assorter-v1".to_string(),
            assorter_upper_bound: RationalValue {
                numerator: 1,
                denominator: 1,
            },
            winner_selection_id: Some("cand-a".to_string()),
            loser_selection_id: Some("cand-b".to_string()),
        }],
        sample_steps,
        decision,
        source_refs: Vec::new(),
    }
}

fn copy_dir_all(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let dst_path = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dst_path)?;
        } else {
            std::fs::copy(entry.path(), dst_path)?;
        }
    }
    Ok(())
}
