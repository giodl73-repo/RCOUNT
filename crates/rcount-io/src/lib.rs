use rcount_core::{
    package_content_hash, verify_jurisdiction_total, verify_package, AuditAlgorithmDecision,
    AuditAlgorithmRun, AuditAssertion, AuditAssertionKind, AuditSamplingMode, BatchKind,
    BatchManifest, Contest, CountStatus, RcountPackage, ReportingUnit, ReportingUnitKind,
    Selection, SelectionKind, SelectionTotal, StatusEvent, Summary,
    ATHENA_BALLOT_POLLING_METHOD_ID, MINERVA_BALLOT_POLLING_METHOD_ID, RCOUNT_VERSION,
    SOURCE_HASH_PREFIX,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RcountIoError {
    #[error("core error: {0}")]
    Core(#[from] rcount_core::RcountCoreError),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("csv error: {0}")]
    Csv(#[from] csv::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("unsupported RCOUNT version: {0}")]
    UnsupportedVersion(String),
    #[error("manifest content_hash mismatch: declared {declared}, computed {computed}")]
    ContentHashMismatch { declared: String, computed: String },
    #[error("source index is empty")]
    EmptySourceIndex,
    #[error("source path is not package-relative under sources/: {path}")]
    InvalidSourcePath { path: String },
    #[error("source file is missing: {path}")]
    MissingSourceFile { path: String },
    #[error("source hash mismatch for {source_id}: declared {declared}, computed {computed}")]
    SourceHashMismatch {
        source_id: String,
        declared: String,
        computed: String,
    },
    #[error("statement CSV row {row} is missing {field}")]
    MissingStatementCsvField { row: usize, field: String },
    #[error("statement CSV row {row} has invalid {field}: {value}")]
    InvalidStatementCsvField {
        row: usize,
        field: String,
        value: String,
    },
    #[error("statement CSV row {row} conflicts with prior {field} for {id}: {prior} vs {value}")]
    ConflictingStatementCsvField {
        row: usize,
        id: String,
        field: String,
        prior: String,
        value: String,
    },
    #[error("NIST CDF import is missing {field}")]
    MissingNistCdfField { field: String },
    #[error("NIST CDF import has invalid {field}: {value}")]
    InvalidNistCdfField { field: String, value: String },
    #[error("Rhode Island RLA import is missing section {section}")]
    MissingRhodeIslandRlaSection { section: String },
    #[error("Rhode Island RLA import is missing field {field}")]
    MissingRhodeIslandRlaField { field: String },
    #[error("Rhode Island RLA import has invalid field {field}: {value}")]
    InvalidRhodeIslandRlaField { field: String, value: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RcountManifest {
    pub rcount_version: String,
    pub jurisdiction: Jurisdiction,
    pub election: Election,
    pub status: String,
    pub hash_algorithm: String,
    pub content_hash: String,
    pub created_by: CreatedBy,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Jurisdiction {
    pub country: String,
    pub state: String,
    pub county: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Election {
    pub date: String,
    #[serde(rename = "type")]
    pub election_type: String,
    pub scope: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreatedBy {
    pub tool: String,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceIndex {
    pub sources: Vec<SourceEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceEntry {
    pub source_id: String,
    pub path: String,
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageHashes {
    pub package_content_hash: String,
    pub contest_count: usize,
    pub reporting_unit_count: usize,
    pub batch_count: usize,
    pub lineage_count: usize,
    #[serde(default)]
    pub rhist_ref_count: usize,
    #[serde(default)]
    pub rctx_ref_count: usize,
    pub inclusion_proof_count: usize,
    pub cvr_count: usize,
    #[serde(default)]
    pub audit_algorithm_run_count: usize,
    pub rla_audit_count: usize,
    pub manual_audit_count: usize,
    #[serde(default)]
    pub batch_comparison_audit_count: usize,
    pub summary_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceCheck {
    pub source_id: String,
    pub path: String,
    pub sha256: String,
}

pub fn synthetic_summary_basic_manifest(
    package: &RcountPackage,
) -> Result<RcountManifest, RcountIoError> {
    synthetic_manifest(package, "canvassed")
}

pub fn synthetic_canvass_correction_manifest(
    package: &RcountPackage,
) -> Result<RcountManifest, RcountIoError> {
    synthetic_manifest(package, "canvassed")
}

fn synthetic_manifest(
    package: &RcountPackage,
    status: &str,
) -> Result<RcountManifest, RcountIoError> {
    Ok(RcountManifest {
        rcount_version: RCOUNT_VERSION.to_string(),
        jurisdiction: Jurisdiction {
            country: "US".to_string(),
            state: "SYN".to_string(),
            county: "SYN-COUNTY-1".to_string(),
        },
        election: Election {
            date: "2024-11-05".to_string(),
            election_type: "general".to_string(),
            scope: "synthetic-county".to_string(),
        },
        status: status.to_string(),
        hash_algorithm: "sha256".to_string(),
        content_hash: package_content_hash(package)?,
        created_by: CreatedBy {
            tool: "rcount-io-example".to_string(),
            version: RCOUNT_VERSION.to_string(),
        },
    })
}

pub fn write_package_dir(
    dir: &Path,
    manifest: &RcountManifest,
    package: &RcountPackage,
) -> Result<(), RcountIoError> {
    fs::create_dir_all(dir.join("sources"))?;
    fs::create_dir_all(dir.join("normalized"))?;
    fs::create_dir_all(dir.join("reconciliation"))?;
    fs::create_dir_all(dir.join("status"))?;
    fs::create_dir_all(dir.join("proofs"))?;
    fs::create_dir_all(dir.join("audits"))?;
    fs::create_dir_all(dir.join("transcripts"))?;

    let computed = package_content_hash(package)?;
    let mut manifest = manifest.clone();
    manifest.content_hash = computed.clone();

    write_json_pretty(&dir.join("manifest.json"), &manifest)?;
    let source_entry = write_synthetic_source_export(dir, package)?;
    write_json_pretty(
        &dir.join("sources").join("source-index.json"),
        &SourceIndex {
            sources: vec![source_entry],
        },
    )?;
    write_ndjson(
        &dir.join("normalized").join("contests.ndjson"),
        &package.contests,
    )?;
    write_ndjson(
        &dir.join("normalized").join("reporting-units.ndjson"),
        &package.reporting_units,
    )?;
    write_ndjson(
        &dir.join("normalized").join("batches.ndjson"),
        &package.batches,
    )?;
    write_ndjson(
        &dir.join("normalized").join("lineage.ndjson"),
        &package.lineage,
    )?;
    write_ndjson(
        &dir.join("normalized").join("rhist-refs.ndjson"),
        &package.rhist_refs,
    )?;
    write_ndjson(
        &dir.join("normalized").join("rctx-refs.ndjson"),
        &package.rctx_refs,
    )?;
    write_ndjson(
        &dir.join("proofs").join("inclusion-proofs.ndjson"),
        &package.inclusion_proofs,
    )?;
    write_ndjson(&dir.join("normalized").join("cvr.ndjson"), &package.cvr)?;
    write_ndjson(
        &dir.join("audits").join("algorithm-runs.ndjson"),
        &package.audit_algorithm_runs,
    )?;
    write_ndjson(&dir.join("audits").join("rla.ndjson"), &package.rla_audits)?;
    write_ndjson(
        &dir.join("audits").join("manual.ndjson"),
        &package.manual_audits,
    )?;
    write_ndjson(
        &dir.join("audits").join("batch-comparison.ndjson"),
        &package.batch_comparison_audits,
    )?;
    write_ndjson(
        &dir.join("normalized").join("summaries.ndjson"),
        &package.summaries,
    )?;
    write_lines(
        &dir.join("reconciliation").join("equations.ndjson"),
        &[
            r#"{"equation_id":"contest_selection_sum","status":"declared"}"#,
            r#"{"equation_id":"jurisdiction_contest_total","status":"declared"}"#,
            r#"{"equation_id":"batch_summary_total","status":"declared"}"#,
            r#"{"equation_id":"lineage_conservation","status":"declared"}"#,
            r#"{"equation_id":"rhist_reference_declared","status":"declared"}"#,
            r#"{"equation_id":"rctx_reference_declared","status":"declared"}"#,
            r#"{"equation_id":"status_event_declared","status":"declared"}"#,
            r#"{"equation_id":"canvass_correction_event","status":"declared"}"#,
            r#"{"equation_id":"cvr_summary_total","status":"declared"}"#,
            r#"{"equation_id":"rla_sampler_replay","status":"declared"}"#,
            r#"{"equation_id":"rla_margin_metadata","status":"declared"}"#,
            r#"{"equation_id":"rla_stopping_rule","status":"declared"}"#,
            r#"{"equation_id":"manual_audit_reconciliation","status":"declared"}"#,
            r#"{"equation_id":"batch_comparison_overstatement","status":"declared"}"#,
        ],
    )?;
    write_ndjson(
        &dir.join("status").join("events.ndjson"),
        &package.status_events,
    )?;
    write_json_pretty(
        &dir.join("proofs").join("package-hashes.json"),
        &PackageHashes {
            package_content_hash: computed,
            contest_count: package.contests.len(),
            reporting_unit_count: package.reporting_units.len(),
            batch_count: package.batches.len(),
            lineage_count: package.lineage.len(),
            rhist_ref_count: package.rhist_refs.len(),
            rctx_ref_count: package.rctx_refs.len(),
            inclusion_proof_count: package.inclusion_proofs.len(),
            cvr_count: package.cvr.len(),
            audit_algorithm_run_count: package.audit_algorithm_runs.len(),
            rla_audit_count: package.rla_audits.len(),
            manual_audit_count: package.manual_audits.len(),
            batch_comparison_audit_count: package.batch_comparison_audits.len(),
            summary_count: package.summaries.len(),
        },
    )?;
    write_json_pretty(
        &dir.join("transcripts").join("verify-transcript.json"),
        &serde_json::json!({
            "status": "generated-fixture",
            "verifier": "rcount-io",
            "checks": ["contest_selection_sum", "jurisdiction_contest_total"]
        }),
    )?;
    Ok(())
}

pub fn read_package_dir(dir: &Path) -> Result<(RcountManifest, RcountPackage), RcountIoError> {
    let manifest: RcountManifest = read_json(&dir.join("manifest.json"))?;
    if manifest.rcount_version != RCOUNT_VERSION {
        return Err(RcountIoError::UnsupportedVersion(manifest.rcount_version));
    }
    let package = RcountPackage {
        rcount_version: manifest.rcount_version.clone(),
        contests: read_ndjson(&dir.join("normalized").join("contests.ndjson"))?,
        reporting_units: read_ndjson(&dir.join("normalized").join("reporting-units.ndjson"))?,
        batches: read_optional_ndjson(&dir.join("normalized").join("batches.ndjson"))?,
        lineage: read_optional_ndjson(&dir.join("normalized").join("lineage.ndjson"))?,
        rhist_refs: read_optional_ndjson(&dir.join("normalized").join("rhist-refs.ndjson"))?,
        rctx_refs: read_optional_ndjson(&dir.join("normalized").join("rctx-refs.ndjson"))?,
        inclusion_proofs: read_optional_ndjson(
            &dir.join("proofs").join("inclusion-proofs.ndjson"),
        )?,
        cvr: read_optional_ndjson(&dir.join("normalized").join("cvr.ndjson"))?,
        audit_algorithm_runs: read_optional_ndjson(
            &dir.join("audits").join("algorithm-runs.ndjson"),
        )?,
        rla_audits: read_optional_ndjson(&dir.join("audits").join("rla.ndjson"))?,
        manual_audits: read_optional_ndjson(&dir.join("audits").join("manual.ndjson"))?,
        batch_comparison_audits: read_optional_ndjson(
            &dir.join("audits").join("batch-comparison.ndjson"),
        )?,
        summaries: read_ndjson(&dir.join("normalized").join("summaries.ndjson"))?,
        status_events: read_ndjson(&dir.join("status").join("events.ndjson"))?,
    };
    let computed = package_content_hash(&package)?;
    if manifest.content_hash != computed {
        return Err(RcountIoError::ContentHashMismatch {
            declared: manifest.content_hash,
            computed,
        });
    }
    Ok((manifest, package))
}

pub fn read_source_index(dir: &Path) -> Result<SourceIndex, RcountIoError> {
    read_json(&dir.join("sources").join("source-index.json"))
}

pub fn verify_source_index(dir: &Path) -> Result<Vec<SourceCheck>, RcountIoError> {
    let index = read_source_index(dir)?;
    if index.sources.is_empty() {
        return Err(RcountIoError::EmptySourceIndex);
    }

    let mut checks = Vec::new();
    for source in index.sources {
        let path = package_relative_source_path(&source.path)?;
        let full_path = dir.join(&path);
        if !full_path.exists() {
            return Err(RcountIoError::MissingSourceFile {
                path: source.path.clone(),
            });
        }
        let computed = source_file_hash(&full_path)?;
        if computed != source.sha256 {
            return Err(RcountIoError::SourceHashMismatch {
                source_id: source.source_id,
                declared: source.sha256,
                computed,
            });
        }
        checks.push(SourceCheck {
            source_id: source.source_id,
            path: source.path,
            sha256: computed,
        });
    }
    Ok(checks)
}

pub fn source_file_hash(path: &Path) -> Result<String, RcountIoError> {
    Ok(source_bytes_hash(&fs::read(path)?))
}

pub fn verify_summary_basic_dir(dir: &Path) -> Result<(), RcountIoError> {
    let (_, package) = read_package_dir(dir)?;
    verify_package(&package)?;
    verify_jurisdiction_total("syn-2024-mayor", "syn:jurisdiction:SYN", &package.summaries)?;
    Ok(())
}

pub fn default_summary_basic_docs_dir() -> PathBuf {
    PathBuf::from("docs")
        .join("examples")
        .join("rcount-golden-packages")
        .join("summary-basic")
}

pub fn default_canvass_correction_docs_dir() -> PathBuf {
    PathBuf::from("docs")
        .join("examples")
        .join("rcount-golden-packages")
        .join("canvass-correction")
}

pub fn default_bad_selection_sum_docs_dir() -> PathBuf {
    PathBuf::from("docs")
        .join("examples")
        .join("rcount-golden-packages")
        .join("bad-selection-sum")
}

pub fn default_mail_batch_added_docs_dir() -> PathBuf {
    PathBuf::from("docs")
        .join("examples")
        .join("rcount-golden-packages")
        .join("mail-batch-added")
}

pub fn default_missing_batch_docs_dir() -> PathBuf {
    PathBuf::from("docs")
        .join("examples")
        .join("rcount-golden-packages")
        .join("missing-batch")
}

pub fn default_precinct_split_lineage_docs_dir() -> PathBuf {
    PathBuf::from("docs")
        .join("examples")
        .join("rcount-golden-packages")
        .join("precinct-split-lineage")
}

pub fn default_bad_lineage_docs_dir() -> PathBuf {
    PathBuf::from("docs")
        .join("examples")
        .join("rcount-golden-packages")
        .join("bad-lineage")
}

pub fn default_privacy_inclusion_sketch_docs_dir() -> PathBuf {
    PathBuf::from("docs")
        .join("examples")
        .join("rcount-golden-packages")
        .join("privacy-inclusion-sketch")
}

pub fn default_choice_bearing_proof_docs_dir() -> PathBuf {
    PathBuf::from("docs")
        .join("examples")
        .join("rcount-golden-packages")
        .join("choice-bearing-proof")
}

pub fn default_cvr_summary_docs_dir() -> PathBuf {
    PathBuf::from("docs")
        .join("examples")
        .join("rcount-golden-packages")
        .join("cvr-summary")
}

pub fn default_bad_cvr_summary_docs_dir() -> PathBuf {
    PathBuf::from("docs")
        .join("examples")
        .join("rcount-golden-packages")
        .join("bad-cvr-summary")
}

pub fn default_rla_replay_docs_dir() -> PathBuf {
    PathBuf::from("docs")
        .join("examples")
        .join("rcount-golden-packages")
        .join("rla-replay")
}

pub fn default_bad_rla_replay_docs_dir() -> PathBuf {
    PathBuf::from("docs")
        .join("examples")
        .join("rcount-golden-packages")
        .join("bad-rla-replay")
}

pub fn default_rla_stopping_docs_dir() -> PathBuf {
    PathBuf::from("docs")
        .join("examples")
        .join("rcount-golden-packages")
        .join("rla-stopping")
}

pub fn default_bad_rla_stopping_docs_dir() -> PathBuf {
    PathBuf::from("docs")
        .join("examples")
        .join("rcount-golden-packages")
        .join("bad-rla-stopping")
}

pub fn default_rla_discrepancy_docs_dir() -> PathBuf {
    PathBuf::from("docs")
        .join("examples")
        .join("rcount-golden-packages")
        .join("rla-discrepancy")
}

pub fn default_bad_rla_discrepancy_docs_dir() -> PathBuf {
    PathBuf::from("docs")
        .join("examples")
        .join("rcount-golden-packages")
        .join("bad-rla-discrepancy")
}

pub fn default_rla_margin_docs_dir() -> PathBuf {
    PathBuf::from("docs")
        .join("examples")
        .join("rcount-golden-packages")
        .join("rla-margin")
}

pub fn default_bad_rla_margin_docs_dir() -> PathBuf {
    PathBuf::from("docs")
        .join("examples")
        .join("rcount-golden-packages")
        .join("bad-rla-margin")
}

pub fn default_rla_statistical_docs_dir() -> PathBuf {
    PathBuf::from("docs")
        .join("examples")
        .join("rcount-golden-packages")
        .join("rla-statistical")
}

pub fn default_bad_rla_statistical_docs_dir() -> PathBuf {
    PathBuf::from("docs")
        .join("examples")
        .join("rcount-golden-packages")
        .join("bad-rla-statistical")
}

pub fn default_colorado_rla_docs_dir() -> PathBuf {
    PathBuf::from("docs")
        .join("examples")
        .join("rcount-golden-packages")
        .join("colorado-rla")
}

pub fn default_bad_colorado_rla_docs_dir() -> PathBuf {
    PathBuf::from("docs")
        .join("examples")
        .join("rcount-golden-packages")
        .join("bad-colorado-rla")
}

pub fn default_california_rla_docs_dir() -> PathBuf {
    PathBuf::from("docs")
        .join("examples")
        .join("rcount-golden-packages")
        .join("california-rla")
}

pub fn default_bad_california_rla_docs_dir() -> PathBuf {
    PathBuf::from("docs")
        .join("examples")
        .join("rcount-golden-packages")
        .join("bad-california-rla")
}

pub fn default_manual_audit_docs_dir() -> PathBuf {
    PathBuf::from("docs")
        .join("examples")
        .join("rcount-golden-packages")
        .join("manual-audit")
}

pub fn default_bad_manual_audit_docs_dir() -> PathBuf {
    PathBuf::from("docs")
        .join("examples")
        .join("rcount-golden-packages")
        .join("bad-manual-audit")
}

#[derive(Debug, Deserialize)]
struct StatementCsvRow {
    contest_id: String,
    contest_title: String,
    vote_for: String,
    selection_id: String,
    selection_label: String,
    selection_kind: String,
    reporting_unit_id: String,
    reporting_unit_kind: String,
    parent_jurisdiction: String,
    status: String,
    votes: String,
    undervotes: String,
    overvotes: String,
    blank_contests: String,
    counted_ballots: String,
}

#[derive(Debug, Clone)]
struct SummaryAccumulator {
    contest_id: String,
    reporting_unit_id: String,
    status: CountStatus,
    totals: Vec<SelectionTotal>,
    seen_selection_ids: BTreeSet<String>,
    undervotes: i64,
    overvotes: i64,
    blank_contests: i64,
    counted_ballots: i64,
}

#[derive(Debug, Deserialize)]
struct RhodeIslandManifestRow {
    #[serde(rename = "Batch Name")]
    batch_name: String,
    #[serde(rename = "Number of Ballots")]
    number_of_ballots: String,
    #[serde(rename = "Container")]
    container: String,
    #[serde(rename = "Tabulator")]
    tabulator: String,
}

#[derive(Debug, Deserialize)]
struct RhodeIslandRetrievalRow {
    #[serde(rename = "Container")]
    container: String,
    #[serde(rename = "Tabulator")]
    tabulator: String,
    #[serde(rename = "Batch Name")]
    batch_name: String,
    #[serde(rename = "Ballot Number")]
    ballot_number: String,
    #[serde(rename = "Ticket Numbers")]
    ticket_numbers: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RhodeIslandRlaSourceSummary {
    pub adapter_id: String,
    pub contest_id: String,
    pub audit_method: String,
    pub risk_limit_ppm: u32,
    pub public_seed: String,
    pub declared_sample_size: u32,
    pub sampled_ballot_rows: usize,
    pub retrieval_rows: usize,
    pub claim_boundary: Vec<String>,
}

/// Imports a deliberately small statement-of-votes CSV into the neutral RCOUNT
/// model. This adapter is the V.9 fixture surface: one row per
/// contest/reporting-unit/selection total, plus repeated residual columns.
pub fn import_statement_csv(path: &Path) -> Result<RcountPackage, RcountIoError> {
    let mut reader = csv::Reader::from_path(path)?;
    let mut contests: BTreeMap<String, Contest> = BTreeMap::new();
    let mut reporting_units: BTreeMap<String, ReportingUnit> = BTreeMap::new();
    let mut summaries: BTreeMap<(String, String, CountStatus), SummaryAccumulator> =
        BTreeMap::new();

    for (index, row) in reader.deserialize::<StatementCsvRow>().enumerate() {
        let row_number = index + 2;
        let row = row?;
        let contest_id = required(row_number, "contest_id", row.contest_id)?;
        let contest_title = required(row_number, "contest_title", row.contest_title)?;
        let vote_for = parse_u32(row_number, "vote_for", row.vote_for)?;
        let selection_id = required(row_number, "selection_id", row.selection_id)?;
        let selection_label = required(row_number, "selection_label", row.selection_label)?;
        let selection_kind = parse_selection_kind(row_number, row.selection_kind)?;
        let reporting_unit_id = required(row_number, "reporting_unit_id", row.reporting_unit_id)?;
        let reporting_unit_kind = parse_reporting_unit_kind(row_number, row.reporting_unit_kind)?;
        let parent_jurisdiction =
            required(row_number, "parent_jurisdiction", row.parent_jurisdiction)?;
        let status = parse_count_status(row_number, row.status)?;
        let votes = parse_i64(row_number, "votes", row.votes)?;
        let undervotes = parse_i64(row_number, "undervotes", row.undervotes)?;
        let overvotes = parse_i64(row_number, "overvotes", row.overvotes)?;
        let blank_contests = parse_i64(row_number, "blank_contests", row.blank_contests)?;
        let counted_ballots = parse_i64(row_number, "counted_ballots", row.counted_ballots)?;

        let contest = contests.entry(contest_id.clone()).or_insert(Contest {
            contest_id: contest_id.clone(),
            title: contest_title.clone(),
            vote_for,
            selections: Vec::new(),
        });
        require_same(
            row_number,
            &contest_id,
            "contest_title",
            &contest.title,
            &contest_title,
        )?;
        require_same(
            row_number,
            &contest_id,
            "vote_for",
            &contest.vote_for.to_string(),
            &vote_for.to_string(),
        )?;
        if let Some(selection) = contest
            .selections
            .iter()
            .find(|selection| selection.selection_id == selection_id)
        {
            require_same(
                row_number,
                &selection_id,
                "selection_label",
                &selection.label,
                &selection_label,
            )?;
            if selection.kind != selection_kind {
                return Err(RcountIoError::ConflictingStatementCsvField {
                    row: row_number,
                    id: selection_id.clone(),
                    field: "selection_kind".to_string(),
                    prior: format!("{:?}", selection.kind),
                    value: format!("{selection_kind:?}"),
                });
            }
        } else {
            contest.selections.push(Selection {
                selection_id: selection_id.clone(),
                kind: selection_kind,
                label: selection_label,
            });
        }

        let unit = reporting_units
            .entry(reporting_unit_id.clone())
            .or_insert(ReportingUnit {
                reporting_unit_id: reporting_unit_id.clone(),
                kind: reporting_unit_kind.clone(),
                parent_jurisdiction: parent_jurisdiction.clone(),
                source_ids: vec!["source:statement-csv".to_string()],
                valid_from: None,
                valid_to: None,
            });
        if unit.kind != reporting_unit_kind {
            return Err(RcountIoError::ConflictingStatementCsvField {
                row: row_number,
                id: reporting_unit_id,
                field: "reporting_unit_kind".to_string(),
                prior: format!("{:?}", unit.kind),
                value: format!("{reporting_unit_kind:?}"),
            });
        }
        require_same(
            row_number,
            &unit.reporting_unit_id,
            "parent_jurisdiction",
            &unit.parent_jurisdiction,
            &parent_jurisdiction,
        )?;

        let key = (contest_id.clone(), reporting_unit_id.clone(), status);
        let summary = summaries.entry(key).or_insert(SummaryAccumulator {
            contest_id,
            reporting_unit_id,
            status,
            totals: Vec::new(),
            seen_selection_ids: BTreeSet::new(),
            undervotes,
            overvotes,
            blank_contests,
            counted_ballots,
        });
        require_same(
            row_number,
            &summary.reporting_unit_id,
            "undervotes",
            &summary.undervotes.to_string(),
            &undervotes.to_string(),
        )?;
        require_same(
            row_number,
            &summary.reporting_unit_id,
            "overvotes",
            &summary.overvotes.to_string(),
            &overvotes.to_string(),
        )?;
        require_same(
            row_number,
            &summary.reporting_unit_id,
            "blank_contests",
            &summary.blank_contests.to_string(),
            &blank_contests.to_string(),
        )?;
        require_same(
            row_number,
            &summary.reporting_unit_id,
            "counted_ballots",
            &summary.counted_ballots.to_string(),
            &counted_ballots.to_string(),
        )?;
        if summary.seen_selection_ids.insert(selection_id.clone()) {
            summary.totals.push(SelectionTotal {
                selection_id,
                votes,
            });
        } else {
            return Err(RcountIoError::ConflictingStatementCsvField {
                row: row_number,
                id: summary.reporting_unit_id.clone(),
                field: "selection_id".to_string(),
                prior: "already present".to_string(),
                value: selection_id,
            });
        }
    }

    Ok(RcountPackage {
        rcount_version: RCOUNT_VERSION.to_string(),
        contests: contests.into_values().collect(),
        reporting_units: reporting_units.into_values().collect(),
        batches: Vec::new(),
        lineage: Vec::new(),
        rhist_refs: Vec::new(),
        rctx_refs: Vec::new(),
        inclusion_proofs: Vec::new(),
        cvr: Vec::new(),
        audit_algorithm_runs: Vec::new(),
        rla_audits: Vec::new(),
        manual_audits: Vec::new(),
        batch_comparison_audits: Vec::new(),
        summaries: summaries
            .into_values()
            .map(|summary| Summary {
                contest_id: summary.contest_id,
                reporting_unit_id: summary.reporting_unit_id,
                batch_id: None,
                status: summary.status,
                totals: summary.totals,
                undervotes: summary.undervotes,
                overvotes: summary.overvotes,
                blank_contests: summary.blank_contests,
                counted_ballots: summary.counted_ballots,
            })
            .collect(),
        status_events: Vec::<StatusEvent>::new(),
    })
}

pub fn write_statement_csv_package_dir(
    dir: &Path,
    csv_path: &Path,
    manifest: &RcountManifest,
    package: &RcountPackage,
) -> Result<(), RcountIoError> {
    write_package_dir(dir, manifest, package)?;
    let source_path = PathBuf::from("sources").join("statement-of-votes.csv");
    let bytes = fs::read(csv_path)?;
    fs::write(dir.join(&source_path), &bytes)?;
    let synthetic = dir.join("sources").join("synthetic-summary-export.json");
    if synthetic.exists() {
        fs::remove_file(synthetic)?;
    }
    write_json_pretty(
        &dir.join("sources").join("source-index.json"),
        &SourceIndex {
            sources: vec![SourceEntry {
                source_id: "source:statement-csv".to_string(),
                path: source_path.to_string_lossy().replace('\\', "/"),
                sha256: source_bytes_hash(&bytes),
            }],
        },
    )?;
    Ok(())
}

/// Imports a small NIST Election Results Reporting CDF-style JSON fixture into
/// RCOUNT. This is a first adapter slice, not a complete CDF implementation.
pub fn import_nist_cdf_json(path: &Path) -> Result<RcountPackage, RcountIoError> {
    let value: Value = serde_json::from_slice(&fs::read(path)?)?;
    let report = value.get("ElectionReport").unwrap_or(&value);
    let status = parse_nist_status(
        report
            .get("ResultsStatus")
            .and_then(Value::as_str)
            .unwrap_or("canvassed"),
    )?;

    let mut reporting_units = BTreeMap::new();
    for unit in array_field(report, "GpUnit")? {
        let id = nist_id(unit, "GpUnit")?;
        let kind = unit
            .get("Type")
            .and_then(Value::as_str)
            .map(parse_nist_reporting_unit_kind)
            .transpose()?
            .unwrap_or(ReportingUnitKind::Precinct);
        reporting_units.insert(
            id.clone(),
            ReportingUnit {
                reporting_unit_id: id,
                kind,
                parent_jurisdiction: "nist-cdf".to_string(),
                source_ids: vec!["source:nist-cdf-json".to_string()],
                valid_from: None,
                valid_to: None,
            },
        );
    }

    let elections = array_field(report, "Election")?;
    let mut contests = BTreeMap::new();
    let mut summaries: BTreeMap<(String, String, CountStatus), SummaryAccumulator> =
        BTreeMap::new();

    for election in elections {
        for contest_value in array_field(election, "Contest")? {
            let contest_id = nist_id(contest_value, "Contest")?;
            let contest_title = nist_text(contest_value.get("Name")).unwrap_or(contest_id.clone());
            let vote_for = contest_value
                .get("NumberElected")
                .or_else(|| contest_value.get("VotesAllowed"))
                .and_then(Value::as_u64)
                .unwrap_or(1) as u32;
            let contest = contests.entry(contest_id.clone()).or_insert(Contest {
                contest_id: contest_id.clone(),
                title: contest_title.clone(),
                vote_for,
                selections: Vec::new(),
            });
            require_same(
                0,
                &contest_id,
                "contest_title",
                &contest.title,
                &contest_title,
            )?;

            for selection_value in array_field(contest_value, "ContestSelection")? {
                let selection_id = nist_id(selection_value, "ContestSelection")?;
                let selection_label =
                    nist_text(selection_value.get("Name")).unwrap_or(selection_id.clone());
                let selection_kind = if selection_value
                    .get("IsWriteIn")
                    .and_then(Value::as_bool)
                    .unwrap_or(false)
                {
                    SelectionKind::WriteInBucket
                } else {
                    SelectionKind::Candidate
                };
                if !contest
                    .selections
                    .iter()
                    .any(|selection| selection.selection_id == selection_id)
                {
                    contest.selections.push(Selection {
                        selection_id: selection_id.clone(),
                        kind: selection_kind,
                        label: selection_label,
                    });
                }
                for count in array_field(selection_value, "VoteCounts")? {
                    let reporting_unit_id = nist_gp_unit_ref(count)?;
                    ensure_nist_unit(&mut reporting_units, &reporting_unit_id);
                    let votes = nist_count(count, "Count")?;
                    let summary = summaries
                        .entry((contest_id.clone(), reporting_unit_id.clone(), status))
                        .or_insert(SummaryAccumulator {
                            contest_id: contest_id.clone(),
                            reporting_unit_id,
                            status,
                            totals: Vec::new(),
                            seen_selection_ids: BTreeSet::new(),
                            undervotes: 0,
                            overvotes: 0,
                            blank_contests: 0,
                            counted_ballots: 0,
                        });
                    if summary.seen_selection_ids.insert(selection_id.clone()) {
                        summary.totals.push(SelectionTotal {
                            selection_id: selection_id.clone(),
                            votes,
                        });
                    }
                }
            }

            for other_count in optional_array_field(contest_value, "OtherCounts") {
                let reporting_unit_id = nist_gp_unit_ref(other_count)?;
                ensure_nist_unit(&mut reporting_units, &reporting_unit_id);
                let summary = summaries
                    .entry((contest_id.clone(), reporting_unit_id.clone(), status))
                    .or_insert(SummaryAccumulator {
                        contest_id: contest_id.clone(),
                        reporting_unit_id,
                        status,
                        totals: Vec::new(),
                        seen_selection_ids: BTreeSet::new(),
                        undervotes: 0,
                        overvotes: 0,
                        blank_contests: 0,
                        counted_ballots: 0,
                    });
                summary.undervotes += optional_nist_count(other_count, "Undervotes")?;
                summary.overvotes += optional_nist_count(other_count, "Overvotes")?;
                summary.blank_contests += optional_nist_count(other_count, "BlankVotes")?;
            }
        }
    }

    Ok(RcountPackage {
        rcount_version: RCOUNT_VERSION.to_string(),
        contests: contests.into_values().collect(),
        reporting_units: reporting_units.into_values().collect(),
        batches: Vec::new(),
        lineage: Vec::new(),
        rhist_refs: Vec::new(),
        rctx_refs: Vec::new(),
        inclusion_proofs: Vec::new(),
        cvr: Vec::new(),
        audit_algorithm_runs: Vec::new(),
        rla_audits: Vec::new(),
        manual_audits: Vec::new(),
        batch_comparison_audits: Vec::new(),
        summaries: summaries
            .into_values()
            .map(|mut summary| {
                summary.counted_ballots =
                    summary.totals.iter().map(|total| total.votes).sum::<i64>()
                        + summary.undervotes
                        + summary.overvotes
                        + summary.blank_contests;
                Summary {
                    contest_id: summary.contest_id,
                    reporting_unit_id: summary.reporting_unit_id,
                    batch_id: None,
                    status: summary.status,
                    totals: summary.totals,
                    undervotes: summary.undervotes,
                    overvotes: summary.overvotes,
                    blank_contests: summary.blank_contests,
                    counted_ballots: summary.counted_ballots,
                }
            })
            .collect(),
        status_events: Vec::<StatusEvent>::new(),
    })
}

pub fn write_nist_cdf_package_dir(
    dir: &Path,
    json_path: &Path,
    manifest: &RcountManifest,
    package: &RcountPackage,
) -> Result<(), RcountIoError> {
    write_package_dir(dir, manifest, package)?;
    let source_path = PathBuf::from("sources").join("nist-cdf-results.json");
    let bytes = fs::read(json_path)?;
    fs::write(dir.join(&source_path), &bytes)?;
    let synthetic = dir.join("sources").join("synthetic-summary-export.json");
    if synthetic.exists() {
        fs::remove_file(synthetic)?;
    }
    write_json_pretty(
        &dir.join("sources").join("source-index.json"),
        &SourceIndex {
            sources: vec![SourceEntry {
                source_id: "source:nist-cdf-json".to_string(),
                path: source_path.to_string_lossy().replace('\\', "/"),
                sha256: source_bytes_hash(&bytes),
            }],
        },
    )?;
    Ok(())
}

pub fn import_ri_2024_rep28_ballot_polling_audit(
    audit_report_csv: &Path,
    ballot_manifest_csv: &Path,
    ballot_retrieval_csv: &Path,
) -> Result<RcountPackage, RcountIoError> {
    let report_rows = read_csv_rows(audit_report_csv)?;
    let contest_row = section_data_row(&report_rows, "######## CONTESTS ########")?;
    let settings_row = section_data_row(&report_rows, "######## AUDIT SETTINGS ########")?;
    let rounds_row = section_data_row(&report_rows, "######## ROUNDS ########")?;

    let contest_title = ri_field(contest_row, 0, "contest name")?;
    let vote_for = ri_field(contest_row, 3, "votes allowed")?
        .parse::<u32>()
        .map_err(|_| RcountIoError::InvalidRhodeIslandRlaField {
            field: "votes allowed".to_string(),
            value: contest_row[3].clone(),
        })?;
    let counted_ballots = ri_i64(contest_row, 4, "total ballots cast")?;
    let vote_totals = parse_ri_vote_totals(ri_field(contest_row, 5, "vote totals")?)?;
    let contest_id = "ri-2024-rep-28".to_string();
    let jurisdiction_unit_id = "ri:state:representative-28".to_string();
    let residual_ballots =
        counted_ballots - vote_totals.iter().map(|(_, votes)| votes).sum::<i64>();
    if residual_ballots < 0 {
        return Err(RcountIoError::InvalidRhodeIslandRlaField {
            field: "vote totals".to_string(),
            value: contest_row[5].clone(),
        });
    }

    let risk_limit_ppm = parse_percent_ppm(ri_field(settings_row, 3, "risk limit")?)?;
    let public_seed = normalize_seed(ri_field(settings_row, 4, "random seed")?)?;
    let audit_method = ri_field(settings_row, 2, "audit math type")?;
    if risk_limit_ppm == 0 {
        return Err(RcountIoError::InvalidRhodeIslandRlaField {
            field: "risk limit".to_string(),
            value: settings_row[3].clone(),
        });
    }
    validate_ri_sample_sources(&report_rows, rounds_row, ballot_retrieval_csv)?;

    let mut reporting_units = vec![ReportingUnit {
        reporting_unit_id: jurisdiction_unit_id.clone(),
        kind: ReportingUnitKind::DistrictTotal,
        parent_jurisdiction: "Rhode Island".to_string(),
        source_ids: vec![
            "source:ri-rla-audit-report".to_string(),
            "source:ri-rla-ballot-manifest".to_string(),
        ],
        valid_from: None,
        valid_to: None,
    }];

    let mut batches = Vec::new();
    let mut manifest_reader = csv::Reader::from_path(ballot_manifest_csv)?;
    for (index, row) in manifest_reader
        .deserialize::<RhodeIslandManifestRow>()
        .enumerate()
    {
        let row_number = index + 2;
        let row = row?;
        let batch_name = required_ri_string(row_number, "Batch Name", row.batch_name)?;
        let batch_id = format!("ri:batch:{}", slug_id(&batch_name));
        let ballots = parse_ri_i64_string(row_number, "Number of Ballots", row.number_of_ballots)?;
        let kind = if batch_name.to_ascii_lowercase().starts_with("mb ") {
            BatchKind::Mail
        } else {
            BatchKind::ElectionDay
        };
        let reporting_unit_kind = match kind {
            BatchKind::Mail => ReportingUnitKind::MailBatch,
            _ => ReportingUnitKind::CentralCountBatch,
        };
        reporting_units.push(ReportingUnit {
            reporting_unit_id: batch_id.clone(),
            kind: reporting_unit_kind,
            parent_jurisdiction: "Rhode Island".to_string(),
            source_ids: vec!["source:ri-rla-ballot-manifest".to_string()],
            valid_from: None,
            valid_to: None,
        });
        batches.push(BatchManifest {
            batch_id: batch_id.clone(),
            reporting_unit_id: batch_id,
            kind,
            status: CountStatus::Canvassed,
            accepted_ballots: ballots,
            counted_ballots: ballots,
            rejected_ballots: 0,
            source_refs: vec![
                "source:ri-rla-ballot-manifest".to_string(),
                format!("container:{}", row.container.trim()),
                format!("tabulator:{}", row.tabulator.trim()),
            ],
        });
    }
    let manifest_ballots = batches
        .iter()
        .map(|batch| batch.counted_ballots)
        .sum::<i64>();
    if manifest_ballots != counted_ballots {
        return Err(RcountIoError::InvalidRhodeIslandRlaField {
            field: "manifest ballot total".to_string(),
            value: manifest_ballots.to_string(),
        });
    }

    Ok(RcountPackage {
        rcount_version: RCOUNT_VERSION.to_string(),
        contests: vec![Contest {
            contest_id: contest_id.clone(),
            title: contest_title.to_string(),
            vote_for,
            selections: vote_totals
                .iter()
                .map(|(label, _)| Selection {
                    selection_id: format!("ri-2024-rep28:{}", slug_id(label)),
                    kind: if label.eq_ignore_ascii_case("write-in") {
                        SelectionKind::WriteInBucket
                    } else {
                        SelectionKind::Candidate
                    },
                    label: label.clone(),
                })
                .collect(),
        }],
        reporting_units,
        batches,
        lineage: Vec::new(),
        rhist_refs: Vec::new(),
        rctx_refs: Vec::new(),
        inclusion_proofs: Vec::new(),
        cvr: Vec::new(),
        audit_algorithm_runs: ri_ballot_polling_algorithm_runs(
            &contest_id,
            audit_method,
            risk_limit_ppm,
            &public_seed,
            &vote_totals,
        ),
        rla_audits: Vec::new(),
        manual_audits: Vec::new(),
        batch_comparison_audits: Vec::new(),
        summaries: vec![Summary {
            contest_id,
            reporting_unit_id: jurisdiction_unit_id,
            batch_id: None,
            status: CountStatus::Canvassed,
            totals: vote_totals
                .into_iter()
                .map(|(label, votes)| SelectionTotal {
                    selection_id: format!("ri-2024-rep28:{}", slug_id(&label)),
                    votes,
                })
                .collect(),
            undervotes: 0,
            overvotes: 0,
            blank_contests: residual_ballots,
            counted_ballots,
        }],
        status_events: Vec::<StatusEvent>::new(),
    })
}

fn ri_ballot_polling_algorithm_runs(
    contest_id: &str,
    audit_method: &str,
    risk_limit_ppm: u32,
    public_seed: &str,
    vote_totals: &[(String, i64)],
) -> Vec<AuditAlgorithmRun> {
    let Some(method_id) = ri_ballot_polling_method_id(audit_method) else {
        return Vec::new();
    };
    let mut ranked = vote_totals
        .iter()
        .filter(|(label, votes)| !label.eq_ignore_ascii_case("write-in") && *votes > 0)
        .collect::<Vec<_>>();
    ranked.sort_by(|(_, left), (_, right)| right.cmp(left));
    let assertions = if ranked.len() >= 2 {
        vec![AuditAssertion {
            assertion_id: "assertion:ri-2024-rep28-top-two".to_string(),
            kind: AuditAssertionKind::PluralityWinnerLoser,
            assorter_id: "minerva-ballot-polling-top-two-v1".to_string(),
            assorter_upper_bound: rcount_core::RationalValue {
                numerator: 1,
                denominator: 1,
            },
            winner_selection_id: Some(format!("ri-2024-rep28:{}", slug_id(&ranked[0].0))),
            loser_selection_id: Some(format!("ri-2024-rep28:{}", slug_id(&ranked[1].0))),
        }]
    } else {
        Vec::new()
    };

    Vec::from([AuditAlgorithmRun {
        run_id: "audit-run:ri-2024-rep28-minerva".to_string(),
        contest_id: contest_id.to_string(),
        method_id: method_id.to_string(),
        sampling_mode: AuditSamplingMode::WithReplacement,
        rcv_elimination_order: Vec::new(),
        risk_limit_ppm: Some(risk_limit_ppm),
        reported_winner_votes: ranked.first().map(|(_, votes)| *votes as u64),
        reported_loser_votes: ranked.get(1).map(|(_, votes)| *votes as u64),
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
        assertions,
        sample_steps: Vec::new(),
        decision: AuditAlgorithmDecision::Boundary,
        source_refs: vec![
            "source:ri-rla-audit-report".to_string(),
            "source:ri-rla-ballot-retrieval".to_string(),
            format!("public_seed:{public_seed}"),
        ],
    }])
}

fn ri_ballot_polling_method_id(audit_method: &str) -> Option<&'static str> {
    match audit_method.trim().to_ascii_uppercase().as_str() {
        "MINERVA" => Some(MINERVA_BALLOT_POLLING_METHOD_ID),
        "ATHENA" => Some(ATHENA_BALLOT_POLLING_METHOD_ID),
        _ => None,
    }
}

pub fn ri_2024_rep28_manifest(package: &RcountPackage) -> Result<RcountManifest, RcountIoError> {
    Ok(RcountManifest {
        rcount_version: RCOUNT_VERSION.to_string(),
        jurisdiction: Jurisdiction {
            country: "US".to_string(),
            state: "RI".to_string(),
            county: "statewide".to_string(),
        },
        election: Election {
            date: "2024-11-05".to_string(),
            election_type: "general".to_string(),
            scope: "state-representative-district-28".to_string(),
        },
        status: "canvassed".to_string(),
        hash_algorithm: "sha256".to_string(),
        content_hash: package_content_hash(package)?,
        created_by: CreatedBy {
            tool: "rcount-io-ri-rla-adapter".to_string(),
            version: RCOUNT_VERSION.to_string(),
        },
    })
}

pub fn ri_2024_rep28_source_summary(
    audit_report_csv: &Path,
    ballot_retrieval_csv: &Path,
) -> Result<RhodeIslandRlaSourceSummary, RcountIoError> {
    let report_rows = read_csv_rows(audit_report_csv)?;
    let settings_row = section_data_row(&report_rows, "######## AUDIT SETTINGS ########")?;
    let rounds_row = section_data_row(&report_rows, "######## ROUNDS ########")?;
    let sampled_ballots = ri_sampled_ballot_keys(&report_rows)?;
    let retrieval_ballots = ri_retrieval_ballot_keys(ballot_retrieval_csv)?;
    Ok(RhodeIslandRlaSourceSummary {
        adapter_id: "ri-2024-rep28-ballot-polling-v1".to_string(),
        contest_id: "ri-2024-rep-28".to_string(),
        audit_method: ri_field(settings_row, 2, "audit math type")?.to_string(),
        risk_limit_ppm: parse_percent_ppm(ri_field(settings_row, 3, "risk limit")?)?,
        public_seed: normalize_seed(ri_field(settings_row, 4, "random seed")?)?,
        declared_sample_size: ri_u32(rounds_row, 3, "sample size")?,
        sampled_ballot_rows: sampled_ballots.len(),
        retrieval_rows: retrieval_ballots.len(),
        claim_boundary: vec![
            "source rows are preserved and hashed".to_string(),
            "audit report sampled-ballot rows match retrieval CSV rows by ballot key".to_string(),
            "retrieval row count does not exceed declared sample size".to_string(),
            "Minerva risk calculation is recorded but not replayed".to_string(),
            "ballot-level human observations are not independently verified".to_string(),
        ],
    })
}

pub fn write_ri_2024_rep28_package_dir(
    dir: &Path,
    audit_report_csv: &Path,
    ballot_manifest_csv: &Path,
    ballot_retrieval_csv: &Path,
    manifest: &RcountManifest,
    package: &RcountPackage,
) -> Result<(), RcountIoError> {
    write_package_dir(dir, manifest, package)?;
    let synthetic = dir.join("sources").join("synthetic-summary-export.json");
    if synthetic.exists() {
        fs::remove_file(synthetic)?;
    }
    let sources = [
        (
            "source:ri-rla-audit-report",
            "ri-2024-rep28-audit-report.csv",
            audit_report_csv,
        ),
        (
            "source:ri-rla-ballot-manifest",
            "ri-2024-rep28-ballot-manifest.csv",
            ballot_manifest_csv,
        ),
        (
            "source:ri-rla-ballot-retrieval",
            "ri-2024-rep28-ballot-retrieval.csv",
            ballot_retrieval_csv,
        ),
    ];
    let mut entries = Vec::new();
    for (source_id, file_name, path) in sources {
        let source_path = PathBuf::from("sources").join(file_name);
        let bytes = fs::read(path)?;
        fs::write(dir.join(&source_path), &bytes)?;
        entries.push(SourceEntry {
            source_id: source_id.to_string(),
            path: source_path.to_string_lossy().replace('\\', "/"),
            sha256: source_bytes_hash(&bytes),
        });
    }
    write_json_pretty(
        &dir.join("sources").join("source-index.json"),
        &SourceIndex { sources: entries },
    )?;
    write_json_pretty(
        &dir.join("transcripts")
            .join("ri-2024-rep28-source-summary.json"),
        &ri_2024_rep28_source_summary(audit_report_csv, ballot_retrieval_csv)?,
    )?;
    Ok(())
}

fn required(row: usize, field: &str, value: String) -> Result<String, RcountIoError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(RcountIoError::MissingStatementCsvField {
            row,
            field: field.to_string(),
        });
    }
    Ok(trimmed.to_string())
}

fn read_csv_rows(path: &Path) -> Result<Vec<Vec<String>>, RcountIoError> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .from_path(path)?;
    reader
        .records()
        .map(|record| {
            Ok(record?
                .iter()
                .map(|field| field.trim().to_string())
                .collect())
        })
        .collect()
}

fn section_data_row<'a>(
    rows: &'a [Vec<String>],
    section: &str,
) -> Result<&'a Vec<String>, RcountIoError> {
    let index = rows
        .iter()
        .position(|row| row.first().map_or(false, |field| field == section))
        .ok_or_else(|| RcountIoError::MissingRhodeIslandRlaSection {
            section: section.to_string(),
        })?;
    rows.get(index + 2)
        .ok_or_else(|| RcountIoError::MissingRhodeIslandRlaSection {
            section: section.to_string(),
        })
}

fn ri_field<'a>(row: &'a [String], index: usize, field: &str) -> Result<&'a str, RcountIoError> {
    row.get(index)
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| RcountIoError::MissingRhodeIslandRlaField {
            field: field.to_string(),
        })
}

fn ri_i64(row: &[String], index: usize, field: &str) -> Result<i64, RcountIoError> {
    let value = ri_field(row, index, field)?;
    value
        .parse::<i64>()
        .map_err(|_| RcountIoError::InvalidRhodeIslandRlaField {
            field: field.to_string(),
            value: value.to_string(),
        })
}

fn ri_u32(row: &[String], index: usize, field: &str) -> Result<u32, RcountIoError> {
    let value = ri_field(row, index, field)?;
    value
        .parse::<u32>()
        .map_err(|_| RcountIoError::InvalidRhodeIslandRlaField {
            field: field.to_string(),
            value: value.to_string(),
        })
}

fn validate_ri_sample_sources(
    report_rows: &[Vec<String>],
    rounds_row: &[String],
    ballot_retrieval_csv: &Path,
) -> Result<(), RcountIoError> {
    let declared_sample_size = ri_u32(rounds_row, 3, "sample size")?;
    let sampled_ballots = ri_sampled_ballot_keys(report_rows)?;
    let retrieval_ballots = ri_retrieval_ballot_keys(ballot_retrieval_csv)?;
    if sampled_ballots.len() != retrieval_ballots.len() {
        return Err(RcountIoError::InvalidRhodeIslandRlaField {
            field: "sampled ballot row count".to_string(),
            value: format!(
                "report={}, retrieval={}",
                sampled_ballots.len(),
                retrieval_ballots.len()
            ),
        });
    }
    if sampled_ballots.len() > declared_sample_size as usize {
        return Err(RcountIoError::InvalidRhodeIslandRlaField {
            field: "sampled ballot row count".to_string(),
            value: format!(
                "report={} exceeds declared sample size {}",
                sampled_ballots.len(),
                declared_sample_size
            ),
        });
    }
    if sampled_ballots != retrieval_ballots {
        return Err(RcountIoError::InvalidRhodeIslandRlaField {
            field: "sampled ballot keys".to_string(),
            value: "audit report sampled ballots differ from retrieval CSV".to_string(),
        });
    }
    Ok(())
}

fn ri_sampled_ballot_keys(rows: &[Vec<String>]) -> Result<BTreeSet<String>, RcountIoError> {
    let section_index = rows
        .iter()
        .position(|row| {
            row.first()
                .map_or(false, |field| field == "######## SAMPLED BALLOTS ########")
        })
        .ok_or_else(|| RcountIoError::MissingRhodeIslandRlaSection {
            section: "######## SAMPLED BALLOTS ########".to_string(),
        })?;
    let mut keys = BTreeSet::new();
    for row in rows.iter().skip(section_index + 2) {
        if row.first().map_or(true, |field| field.is_empty()) {
            continue;
        }
        if row
            .first()
            .is_some_and(|field| field.starts_with("######## "))
        {
            break;
        }
        let container = ri_field(row, 1, "sampled ballot container")?;
        let tabulator = ri_field(row, 2, "sampled ballot tabulator")?;
        let batch_name = ri_field(row, 3, "sampled ballot batch name")?;
        let ballot_number = ri_field(row, 4, "sampled ballot position")?;
        let _ticket = ri_field(row, 5, "sampled ballot ticket")?
            .strip_prefix("Round 1:")
            .map(str::trim)
            .ok_or_else(|| RcountIoError::InvalidRhodeIslandRlaField {
                field: "sampled ballot ticket".to_string(),
                value: row[5].clone(),
            })?;
        let key = ri_ballot_key(container, tabulator, batch_name, ballot_number);
        if !keys.insert(key.clone()) {
            return Err(RcountIoError::InvalidRhodeIslandRlaField {
                field: "sampled ballot keys".to_string(),
                value: format!("duplicate sampled ballot key {key}"),
            });
        }
    }
    Ok(keys)
}

fn ri_retrieval_ballot_keys(path: &Path) -> Result<BTreeSet<String>, RcountIoError> {
    let mut reader = csv::Reader::from_path(path)?;
    let mut keys = BTreeSet::new();
    for (index, row) in reader.deserialize::<RhodeIslandRetrievalRow>().enumerate() {
        let row_number = index + 2;
        let row = row?;
        let container = required_ri_string(row_number, "Container", row.container)?;
        let tabulator = required_ri_string(row_number, "Tabulator", row.tabulator)?;
        let batch_name = required_ri_string(row_number, "Batch Name", row.batch_name)?;
        let ballot_number = required_ri_string(row_number, "Ballot Number", row.ballot_number)?;
        let _ticket = required_ri_string(row_number, "Ticket Numbers", row.ticket_numbers)?;
        let key = ri_ballot_key(&container, &tabulator, &batch_name, &ballot_number);
        if !keys.insert(key.clone()) {
            return Err(RcountIoError::InvalidRhodeIslandRlaField {
                field: "retrieval ballot keys".to_string(),
                value: format!("duplicate retrieval ballot key {key}"),
            });
        }
    }
    Ok(keys)
}

fn ri_ballot_key(
    container: &str,
    tabulator: &str,
    batch_name: &str,
    ballot_number: &str,
) -> String {
    format!(
        "{}|{}|{}|{}",
        container.trim().trim_start_matches('0'),
        tabulator.trim().trim_start_matches('0'),
        batch_name.trim(),
        ballot_number.trim()
    )
}

fn required_ri_string(row: usize, field: &str, value: String) -> Result<String, RcountIoError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(RcountIoError::MissingRhodeIslandRlaField {
            field: format!("{field} row {row}"),
        });
    }
    Ok(trimmed.to_string())
}

fn parse_ri_i64_string(row: usize, field: &str, value: String) -> Result<i64, RcountIoError> {
    let value = required_ri_string(row, field, value)?;
    value
        .parse::<i64>()
        .map_err(|_| RcountIoError::InvalidRhodeIslandRlaField {
            field: format!("{field} row {row}"),
            value,
        })
}

fn parse_ri_vote_totals(value: &str) -> Result<Vec<(String, i64)>, RcountIoError> {
    value
        .split(';')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(|part| {
            let (label, votes) =
                part.split_once(':')
                    .ok_or_else(|| RcountIoError::InvalidRhodeIslandRlaField {
                        field: "vote totals".to_string(),
                        value: value.to_string(),
                    })?;
            let votes = votes.trim().parse::<i64>().map_err(|_| {
                RcountIoError::InvalidRhodeIslandRlaField {
                    field: "vote totals".to_string(),
                    value: value.to_string(),
                }
            })?;
            Ok((label.trim().to_string(), votes))
        })
        .collect()
}

fn parse_percent_ppm(value: &str) -> Result<u32, RcountIoError> {
    let trimmed = value.trim().trim_end_matches('%');
    let percent =
        trimmed
            .parse::<f64>()
            .map_err(|_| RcountIoError::InvalidRhodeIslandRlaField {
                field: "risk limit".to_string(),
                value: value.to_string(),
            })?;
    if !(0.0..100.0).contains(&percent) {
        return Err(RcountIoError::InvalidRhodeIslandRlaField {
            field: "risk limit".to_string(),
            value: value.to_string(),
        });
    }
    Ok((percent * 10_000.0).round() as u32)
}

fn normalize_seed(value: &str) -> Result<String, RcountIoError> {
    let seed = if value.contains('e') || value.contains('E') {
        let parsed =
            value
                .parse::<f64>()
                .map_err(|_| RcountIoError::InvalidRhodeIslandRlaField {
                    field: "random seed".to_string(),
                    value: value.to_string(),
                })?;
        format!("{parsed:.0}")
    } else {
        value.trim().to_string()
    };
    if seed.chars().all(|ch| ch.is_ascii_digit()) {
        Ok(seed)
    } else {
        Err(RcountIoError::InvalidRhodeIslandRlaField {
            field: "random seed".to_string(),
            value: value.to_string(),
        })
    }
}

fn slug_id(value: &str) -> String {
    let mut slug = String::new();
    let mut last_dash = false;
    for ch in value.chars().flat_map(char::to_lowercase) {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch);
            last_dash = false;
        } else if !last_dash && !slug.is_empty() {
            slug.push('-');
            last_dash = true;
        }
    }
    while slug.ends_with('-') {
        slug.pop();
    }
    slug
}

fn parse_i64(row: usize, field: &str, value: String) -> Result<i64, RcountIoError> {
    let value = required(row, field, value)?;
    value
        .parse::<i64>()
        .map_err(|_| RcountIoError::InvalidStatementCsvField {
            row,
            field: field.to_string(),
            value,
        })
}

fn parse_u32(row: usize, field: &str, value: String) -> Result<u32, RcountIoError> {
    let value = required(row, field, value)?;
    value
        .parse::<u32>()
        .map_err(|_| RcountIoError::InvalidStatementCsvField {
            row,
            field: field.to_string(),
            value,
        })
}

fn parse_selection_kind(row: usize, value: String) -> Result<SelectionKind, RcountIoError> {
    match required(row, "selection_kind", value)?.as_str() {
        "candidate" => Ok(SelectionKind::Candidate),
        "write-in-bucket" => Ok(SelectionKind::WriteInBucket),
        other => Err(RcountIoError::InvalidStatementCsvField {
            row,
            field: "selection_kind".to_string(),
            value: other.to_string(),
        }),
    }
}

fn parse_reporting_unit_kind(
    row: usize,
    value: String,
) -> Result<ReportingUnitKind, RcountIoError> {
    match required(row, "reporting_unit_kind", value)?.as_str() {
        "precinct" => Ok(ReportingUnitKind::Precinct),
        "split-precinct" => Ok(ReportingUnitKind::SplitPrecinct),
        "vote-center" => Ok(ReportingUnitKind::VoteCenter),
        "central-count-batch" => Ok(ReportingUnitKind::CentralCountBatch),
        "mail-batch" => Ok(ReportingUnitKind::MailBatch),
        "provisional-batch" => Ok(ReportingUnitKind::ProvisionalBatch),
        "jurisdiction-total" => Ok(ReportingUnitKind::JurisdictionTotal),
        "district-total" => Ok(ReportingUnitKind::DistrictTotal),
        other => Err(RcountIoError::InvalidStatementCsvField {
            row,
            field: "reporting_unit_kind".to_string(),
            value: other.to_string(),
        }),
    }
}

fn parse_count_status(row: usize, value: String) -> Result<CountStatus, RcountIoError> {
    match required(row, "status", value)?.as_str() {
        "unofficial" => Ok(CountStatus::Unofficial),
        "canvassed" => Ok(CountStatus::Canvassed),
        "recounted" => Ok(CountStatus::Recounted),
        "amended" => Ok(CountStatus::Amended),
        "certified" => Ok(CountStatus::Certified),
        "withdrawn" => Ok(CountStatus::Withdrawn),
        "superseded" => Ok(CountStatus::Superseded),
        other => Err(RcountIoError::InvalidStatementCsvField {
            row,
            field: "status".to_string(),
            value: other.to_string(),
        }),
    }
}

fn require_same(
    row: usize,
    id: &str,
    field: &str,
    prior: &str,
    value: &str,
) -> Result<(), RcountIoError> {
    if prior != value {
        return Err(RcountIoError::ConflictingStatementCsvField {
            row,
            id: id.to_string(),
            field: field.to_string(),
            prior: prior.to_string(),
            value: value.to_string(),
        });
    }
    Ok(())
}

fn array_field<'a>(value: &'a Value, field: &str) -> Result<Vec<&'a Value>, RcountIoError> {
    match value.get(field) {
        Some(Value::Array(values)) => Ok(values.iter().collect()),
        Some(other) => Ok(vec![other]),
        None => Err(RcountIoError::MissingNistCdfField {
            field: field.to_string(),
        }),
    }
}

fn optional_array_field<'a>(value: &'a Value, field: &str) -> Vec<&'a Value> {
    match value.get(field) {
        Some(Value::Array(values)) => values.iter().collect(),
        Some(other) => vec![other],
        None => Vec::new(),
    }
}

fn nist_id(value: &Value, field: &str) -> Result<String, RcountIoError> {
    value
        .get("@id")
        .or_else(|| value.get("id"))
        .or_else(|| value.get("Id"))
        .and_then(Value::as_str)
        .map(ToString::to_string)
        .ok_or_else(|| RcountIoError::MissingNistCdfField {
            field: format!("{field}.@id"),
        })
}

fn nist_text(value: Option<&Value>) -> Option<String> {
    let value = value?;
    if let Some(text) = value.as_str() {
        return Some(text.to_string());
    }
    value
        .get("Text")
        .and_then(Value::as_array)
        .and_then(|texts| texts.first())
        .and_then(|text| text.get("Value"))
        .and_then(Value::as_str)
        .map(ToString::to_string)
}

fn nist_gp_unit_ref(value: &Value) -> Result<String, RcountIoError> {
    value
        .get("GpUnitId")
        .or_else(|| value.get("GpUnit"))
        .or_else(|| value.get("ReportingUnit"))
        .and_then(|field| {
            field
                .as_str()
                .map(ToString::to_string)
                .or_else(|| {
                    field
                        .get("@id")
                        .and_then(Value::as_str)
                        .map(ToString::to_string)
                })
                .or_else(|| {
                    field
                        .get("$ref")
                        .and_then(Value::as_str)
                        .map(ToString::to_string)
                })
        })
        .ok_or_else(|| RcountIoError::MissingNistCdfField {
            field: "GpUnitId".to_string(),
        })
}

fn nist_count(value: &Value, field: &str) -> Result<i64, RcountIoError> {
    value
        .get(field)
        .and_then(Value::as_i64)
        .ok_or_else(|| RcountIoError::MissingNistCdfField {
            field: field.to_string(),
        })
}

fn optional_nist_count(value: &Value, field: &str) -> Result<i64, RcountIoError> {
    match value.get(field) {
        Some(count) => count
            .as_i64()
            .ok_or_else(|| RcountIoError::InvalidNistCdfField {
                field: field.to_string(),
                value: count.to_string(),
            }),
        None => Ok(0),
    }
}

fn parse_nist_status(value: &str) -> Result<CountStatus, RcountIoError> {
    match value {
        "unofficial" | "pre-election" | "election-night" => Ok(CountStatus::Unofficial),
        "canvassed" | "canvass" | "official" => Ok(CountStatus::Canvassed),
        "recounted" | "recount" => Ok(CountStatus::Recounted),
        "amended" => Ok(CountStatus::Amended),
        "certified" => Ok(CountStatus::Certified),
        other => Err(RcountIoError::InvalidNistCdfField {
            field: "ResultsStatus".to_string(),
            value: other.to_string(),
        }),
    }
}

fn parse_nist_reporting_unit_kind(value: &str) -> Result<ReportingUnitKind, RcountIoError> {
    match value {
        "precinct" | "Precinct" => Ok(ReportingUnitKind::Precinct),
        "split-precinct" | "split_precinct" | "SplitPrecinct" => {
            Ok(ReportingUnitKind::SplitPrecinct)
        }
        "vote-center" | "VoteCenter" => Ok(ReportingUnitKind::VoteCenter),
        "district" | "District" => Ok(ReportingUnitKind::DistrictTotal),
        "county" | "state" | "jurisdiction" | "County" | "State" => {
            Ok(ReportingUnitKind::JurisdictionTotal)
        }
        other => Err(RcountIoError::InvalidNistCdfField {
            field: "GpUnit.Type".to_string(),
            value: other.to_string(),
        }),
    }
}

fn ensure_nist_unit(units: &mut BTreeMap<String, ReportingUnit>, reporting_unit_id: &str) {
    units
        .entry(reporting_unit_id.to_string())
        .or_insert(ReportingUnit {
            reporting_unit_id: reporting_unit_id.to_string(),
            kind: ReportingUnitKind::Precinct,
            parent_jurisdiction: "nist-cdf".to_string(),
            source_ids: vec!["source:nist-cdf-json".to_string()],
            valid_from: None,
            valid_to: None,
        });
}

fn write_json_pretty<T: Serialize>(path: &Path, value: &T) -> Result<(), RcountIoError> {
    let bytes = serde_json::to_vec_pretty(value)?;
    fs::write(path, bytes)?;
    Ok(())
}

fn write_synthetic_source_export(
    dir: &Path,
    package: &RcountPackage,
) -> Result<SourceEntry, RcountIoError> {
    let path = PathBuf::from("sources").join("synthetic-summary-export.json");
    let full_path = dir.join(&path);
    let value = serde_json::json!({
        "source_format": "synthetic-summary-export-v1",
        "contest_count": package.contests.len(),
        "reporting_unit_count": package.reporting_units.len(),
        "batch_count": package.batches.len(),
        "lineage_count": package.lineage.len(),
        "rhist_ref_count": package.rhist_refs.len(),
        "rctx_ref_count": package.rctx_refs.len(),
        "inclusion_proof_count": package.inclusion_proofs.len(),
        "cvr_count": package.cvr.len(),
        "rla_audit_count": package.rla_audits.len(),
        "manual_audit_count": package.manual_audits.len(),
        "batch_comparison_audit_count": package.batch_comparison_audits.len(),
        "summary_count": package.summaries.len(),
        "status_event_count": package.status_events.len(),
    });
    let bytes = serde_json::to_vec_pretty(&value)?;
    fs::write(&full_path, &bytes)?;
    Ok(SourceEntry {
        source_id: "source:synthetic-summary-export".to_string(),
        path: path.to_string_lossy().replace('\\', "/"),
        sha256: source_bytes_hash(&bytes),
    })
}

fn source_bytes_hash(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(SOURCE_HASH_PREFIX);
    h.update(bytes);
    format!("sha256:{:x}", h.finalize())
}

fn package_relative_source_path(path: &str) -> Result<PathBuf, RcountIoError> {
    let candidate = Path::new(path);
    if candidate.is_absolute()
        || candidate
            .components()
            .any(|component| matches!(component, std::path::Component::ParentDir))
    {
        return Err(RcountIoError::InvalidSourcePath {
            path: path.to_string(),
        });
    }
    let mut components = candidate.components();
    match components.next() {
        Some(std::path::Component::Normal(first)) if first == "sources" => {}
        _ => {
            return Err(RcountIoError::InvalidSourcePath {
                path: path.to_string(),
            });
        }
    }
    Ok(candidate.to_path_buf())
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, RcountIoError> {
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}

fn write_ndjson<T: Serialize>(path: &Path, records: &[T]) -> Result<(), RcountIoError> {
    let mut file = File::create(path)?;
    for record in records {
        serde_json::to_writer(&mut file, record)?;
        file.write_all(b"\n")?;
    }
    Ok(())
}

fn read_ndjson<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<Vec<T>, RcountIoError> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        records.push(serde_json::from_str(&line)?);
    }
    Ok(records)
}

fn read_optional_ndjson<T: for<'de> Deserialize<'de>>(
    path: &Path,
) -> Result<Vec<T>, RcountIoError> {
    if path.exists() {
        read_ndjson(path)
    } else {
        Ok(Vec::new())
    }
}

fn write_lines(path: &Path, lines: &[&str]) -> Result<(), RcountIoError> {
    let mut file = File::create(path)?;
    for line in lines {
        file.write_all(line.as_bytes())?;
        file.write_all(b"\n")?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rcount_core::{
        synthetic_athena_boundary_package, synthetic_awaire_boundary_package,
        synthetic_bad_california_rla_package, synthetic_bad_colorado_rla_package,
        synthetic_bad_cvr_summary_package, synthetic_bad_lineage_package,
        synthetic_bad_manual_audit_package, synthetic_bad_rla_discrepancy_package,
        synthetic_bad_rla_margin_package, synthetic_bad_rla_replay_package,
        synthetic_bad_rla_statistical_package, synthetic_bad_rla_stopping_package,
        synthetic_bad_selection_sum_package, synthetic_batch_comparison_algorithm_package,
        synthetic_batch_comparison_package, synthetic_batch_size_drift_comparison_package,
        synthetic_bayesian_tabulation_boundary_package, synthetic_california_rla_package,
        synthetic_canvass_correction_package, synthetic_choice_bearing_proof_package,
        synthetic_colorado_rla_package, synthetic_cvr_summary_package,
        synthetic_kaplan_markov_macro_package, synthetic_mail_batch_added_package,
        synthetic_manual_audit_package, synthetic_minerva_multi_round_package,
        synthetic_minerva_round_one_package, synthetic_missing_batch_package,
        synthetic_missing_hand_tally_batch_comparison_package,
        synthetic_precinct_split_lineage_package, synthetic_privacy_inclusion_package,
        synthetic_raire_boundary_package, synthetic_rla_discrepancy_package,
        synthetic_rla_margin_package, synthetic_rla_replay_package,
        synthetic_rla_statistical_package, synthetic_rla_stopping_package,
        synthetic_soba_observable_ballot_boundary_package, synthetic_stratified_hybrid_package,
        synthetic_summary_basic_package, synthetic_summary_basic_package_with_base_references,
        RctxReference, RhistReference, SYN_RCTX_L0_CROSSWALK_HASH, SYN_RHIST_L2_PACKAGE_HASH,
    };

    #[test]
    fn round_trips_synthetic_summary_basic_package() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_summary_basic_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();
        let (decoded_manifest, decoded_package) = read_package_dir(tmp.path()).unwrap();
        assert_eq!(decoded_manifest.content_hash, manifest.content_hash);
        assert_eq!(decoded_package.summaries.len(), 3);
        assert_eq!(verify_source_index(tmp.path()).unwrap().len(), 1);
        verify_summary_basic_dir(tmp.path()).unwrap();
    }

    #[test]
    fn round_trips_synthetic_canvass_correction_package() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_canvass_correction_package();
        let manifest = synthetic_canvass_correction_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();
        let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
        assert_eq!(decoded_package.summaries.len(), 6);
        assert_eq!(decoded_package.status_events.len(), 2);
        assert_eq!(verify_source_index(tmp.path()).unwrap().len(), 1);
    }

    #[test]
    fn round_trips_synthetic_minerva_round_one_package() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_minerva_round_one_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();
        let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
        assert_eq!(decoded_package.audit_algorithm_runs.len(), 1);
        assert_eq!(
            decoded_package.audit_algorithm_runs[0].method_id,
            MINERVA_BALLOT_POLLING_METHOD_ID
        );
        assert_eq!(
            decoded_package.audit_algorithm_runs[0].sample_steps.len(),
            6
        );
    }

    #[test]
    fn round_trips_synthetic_minerva_multi_round_package() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_minerva_multi_round_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();
        let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
        let steps = &decoded_package.audit_algorithm_runs[0].sample_steps;
        assert_eq!(steps.len(), 6);
        assert_eq!(steps[4].round_index, Some(0));
        assert_eq!(steps[5].round_index, Some(1));
    }

    #[test]
    fn round_trips_synthetic_athena_boundary_package() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_athena_boundary_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();
        let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
        assert_eq!(decoded_package.audit_algorithm_runs.len(), 1);
        assert_eq!(
            decoded_package.audit_algorithm_runs[0].method_id,
            ATHENA_BALLOT_POLLING_METHOD_ID
        );
        assert_eq!(
            decoded_package.audit_algorithm_runs[0].decision,
            rcount_core::AuditAlgorithmDecision::Boundary
        );
    }

    #[test]
    fn round_trips_synthetic_stratified_hybrid_package() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_stratified_hybrid_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();
        let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
        let run = decoded_package
            .audit_algorithm_runs
            .iter()
            .find(|run| run.method_id == rcount_core::STRATIFIED_HYBRID_RLA_METHOD_ID)
            .expect("stratified run must round-trip");
        assert_eq!(run.strata.len(), 2);
        assert_eq!(
            run.combining_rule_id.as_deref(),
            Some("suite-nuisance-boundary-v1")
        );
        assert_eq!(
            run.nuisance_parameter,
            Some(rcount_core::RationalValue {
                numerator: 1,
                denominator: 2,
            })
        );
        assert_eq!(run.strata[0].allocation_ppm, Some(500_000));
        assert_eq!(run.strata[1].allocation_ppm, Some(500_000));
    }

    #[test]
    fn round_trips_synthetic_raire_boundary_package() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_raire_boundary_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();
        let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
        let run = &decoded_package.audit_algorithm_runs[0];
        assert_eq!(run.method_id, rcount_core::RAIRE_IRV_METHOD_ID);
        assert_eq!(run.rcv_elimination_order.len(), 3);
        assert_eq!(run.sample_steps[0].ranked_choices[0], "cand-a");
    }

    #[test]
    fn round_trips_synthetic_awaire_boundary_package() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_awaire_boundary_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();
        let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
        assert_eq!(
            decoded_package.audit_algorithm_runs[0].method_id,
            rcount_core::AWAIRE_IRV_METHOD_ID
        );
    }

    #[test]
    fn round_trips_synthetic_bayesian_tabulation_boundary_package() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_bayesian_tabulation_boundary_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();
        let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
        let run = &decoded_package.audit_algorithm_runs[0];
        assert_eq!(
            run.method_id,
            rcount_core::BAYESIAN_TABULATION_AUDIT_METHOD_ID
        );
        assert_eq!(
            run.bayesian_prior_id.as_deref(),
            Some("dirichlet-multinomial-toy-prior-v1")
        );
        assert_eq!(run.posterior_risk_ppm, Some(42_000));
    }

    #[test]
    fn round_trips_synthetic_soba_observable_ballot_boundary_package() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_soba_observable_ballot_boundary_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();
        let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
        let run = &decoded_package.audit_algorithm_runs[0];
        assert_eq!(
            run.method_id,
            rcount_core::SOBA_OBSERVABLE_BALLOT_AUDIT_METHOD_ID
        );
        assert_eq!(decoded_package.inclusion_proofs.len(), 1);
        assert_eq!(
            run.sample_steps[0].sample_unit_id,
            "proof:accepted-token-001"
        );
    }

    #[test]
    fn imports_statement_csv_and_preserves_source_hash() {
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

        let package = import_statement_csv(&csv_path).unwrap();
        verify_package(&package).unwrap();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        let package_dir = tmp.path().join("package");
        write_statement_csv_package_dir(&package_dir, &csv_path, &manifest, &package).unwrap();

        let (_, decoded_package) = read_package_dir(&package_dir).unwrap();
        verify_package(&decoded_package).unwrap();
        let checks = verify_source_index(&package_dir).unwrap();
        assert_eq!(checks[0].source_id, "source:statement-csv");
        assert!(package_dir.join("sources/statement-of-votes.csv").exists());
        assert!(!package_dir
            .join("sources/synthetic-summary-export.json")
            .exists());
    }

    #[test]
    fn imports_nist_cdf_json_and_preserves_source_hash() {
        let tmp = tempfile::tempdir().unwrap();
        let json_path = tmp.path().join("cdf.json");
        std::fs::write(
            &json_path,
            r#"{
  "ElectionReport": {
    "ResultsStatus": "canvassed",
    "GpUnit": [
      {"@id": "syn:precinct:P-001", "Type": "precinct", "Name": {"Text": [{"Value": "P-001"}]}},
      {"@id": "syn:precinct:P-002", "Type": "precinct", "Name": {"Text": [{"Value": "P-002"}]}},
      {"@id": "syn:jurisdiction:SYN", "Type": "county", "Name": {"Text": [{"Value": "SYN County"}]}}
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

        let package = import_nist_cdf_json(&json_path).unwrap();
        verify_package(&package).unwrap();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        let package_dir = tmp.path().join("package");
        write_nist_cdf_package_dir(&package_dir, &json_path, &manifest, &package).unwrap();

        let (_, decoded_package) = read_package_dir(&package_dir).unwrap();
        verify_package(&decoded_package).unwrap();
        let checks = verify_source_index(&package_dir).unwrap();
        assert_eq!(checks[0].source_id, "source:nist-cdf-json");
        assert!(package_dir.join("sources/nist-cdf-results.json").exists());
    }

    #[test]
    fn imports_ri_2024_rep28_rla_sources_and_manifest_batches() {
        let tmp = tempfile::tempdir().unwrap();
        let audit_path = tmp.path().join("audit-report.csv");
        let manifest_path = tmp.path().join("manifest.csv");
        let retrieval_path = tmp.path().join("retrieval.csv");
        std::fs::write(
            &audit_path,
            concat!(
                "######## ELECTION INFO ########,,,,,,,,\n",
                "Organization,Election Name,State,,,,,,\n",
                "Rhode Island,RI General Election 2024 Rep 28,RI,,,,,,\n",
                ",,,,,,,,\n",
                "######## CONTESTS ########,,,,,,,,\n",
                "Contest Name,Targeted?,Number of Winners,Votes Allowed,Total Ballots Cast,Vote Totals,,,\n",
                "Representative 28,Targeted,1,1,13136,Scott Guthrie: 3418; George A. Nardone: 4589; Write-in: 12,,,\n",
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
                "1,0600,0315412524,EV Coventry,39,Round 1: 0.028518425968401157,George A. Nardone,,\n",
            ),
        )
        .unwrap();
        std::fs::write(
            &manifest_path,
            concat!(
                "Batch Name,Number of Ballots,Container,Tabulator\n",
                "EV Coventry,6751,0600,0315412524\n",
                "Coventry 0602,872,0602,0315412769\n",
                "Coventry 0603,612,0603,0315411890\n",
                "Coventry 0604,1055,0604,0315411395\n",
                "Coventry 0611,953,0611,0315412441\n",
                "Coventry 0612,294,0612,0315412579\n",
                "Coventry 0613,538,0613,0315412728\n",
                "Coventry 0614,368,0614,0315412655\n",
                "MB Coventry 1,628,C0017,8520060462\n",
                "MB Coventry 2,671,B0028,8516020237\n",
                "MB Coventry 3,205,A0025,8516020236\n",
                "MB Coventry 4,112,A0039,8516020236\n",
                "MB Coventry 5,65,B0064,8516020237\n",
                "MB Coventry 7,12,B0112,8516020237\n",
            ),
        )
        .unwrap();
        std::fs::write(
            &retrieval_path,
            concat!(
                "Container,Tabulator,Batch Name,Ballot Number,Ticket Numbers,Already Audited,Audit Board\n",
                "0600,0315412524,EV Coventry,39,0.028518425968401157,N,Audit Board #1\n",
            ),
        )
        .unwrap();

        let package =
            import_ri_2024_rep28_ballot_polling_audit(&audit_path, &manifest_path, &retrieval_path)
                .unwrap();
        verify_package(&package).unwrap();
        assert_eq!(package.batches.len(), 14);
        assert_eq!(package.summaries[0].blank_contests, 5117);
        assert_eq!(package.audit_algorithm_runs.len(), 1);
        assert_eq!(
            package.audit_algorithm_runs[0].method_id,
            rcount_core::MINERVA_BALLOT_POLLING_METHOD_ID
        );
        assert_eq!(
            package.audit_algorithm_runs[0].decision,
            rcount_core::AuditAlgorithmDecision::Boundary
        );
        assert_eq!(package.audit_algorithm_runs[0].risk_limit_ppm, Some(90_000));
        assert_eq!(
            package.audit_algorithm_runs[0].reported_winner_votes,
            Some(4589)
        );
        assert_eq!(
            package.audit_algorithm_runs[0].reported_loser_votes,
            Some(3418)
        );

        let manifest = ri_2024_rep28_manifest(&package).unwrap();
        let package_dir = tmp.path().join("ri-package");
        write_ri_2024_rep28_package_dir(
            &package_dir,
            &audit_path,
            &manifest_path,
            &retrieval_path,
            &manifest,
            &package,
        )
        .unwrap();
        let checks = verify_source_index(&package_dir).unwrap();
        assert_eq!(checks.len(), 3);
        assert!(package_dir
            .join("sources/ri-2024-rep28-ballot-retrieval.csv")
            .exists());
        let source_summary = std::fs::read_to_string(
            package_dir.join("transcripts/ri-2024-rep28-source-summary.json"),
        )
        .unwrap();
        assert!(source_summary.contains(r#""sampled_ballot_rows": 1"#));
        assert!(source_summary.contains(r#""retrieval_rows": 1"#));
        assert!(!package_dir
            .join("sources/synthetic-summary-export.json")
            .exists());
    }

    #[test]
    fn ri_2024_rep28_import_rejects_manifest_total_drift() {
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
                "EV Coventry,9,0600,0315412524\n",
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

        assert!(matches!(
            import_ri_2024_rep28_ballot_polling_audit(&audit_path, &manifest_path, &retrieval_path),
            Err(RcountIoError::InvalidRhodeIslandRlaField { .. })
        ));
    }

    #[test]
    fn round_trips_synthetic_mail_batch_added_package() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_mail_batch_added_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();
        let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
        assert_eq!(decoded_package.batches.len(), 3);
        assert_eq!(
            decoded_package
                .summaries
                .iter()
                .filter(|summary| summary.batch_id.is_some())
                .count(),
            3
        );
    }

    #[test]
    fn round_trips_synthetic_missing_batch_package() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_missing_batch_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();
        let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
        assert_eq!(decoded_package.batches.len(), 2);
        assert_eq!(
            decoded_package
                .summaries
                .iter()
                .filter(|summary| summary.batch_id.as_deref() == Some("batch:P-001:late-mail"))
                .count(),
            1
        );
    }

    #[test]
    fn round_trips_synthetic_precinct_split_lineage_package() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_precinct_split_lineage_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();
        let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
        assert_eq!(decoded_package.lineage.len(), 2);
        assert!(decoded_package
            .lineage
            .iter()
            .any(|event| event.lineage_id == "lineage:P-004-split"));
    }

    #[test]
    fn round_trips_rhist_references() {
        let tmp = tempfile::tempdir().unwrap();
        let mut package = synthetic_summary_basic_package();
        package.rhist_refs = vec![RhistReference {
            reference_id: "rhist:real-ri-tract-unchanged".to_string(),
            package_hash: "sha256:ccbddf423aa4ac08b0d45c4ac0b9db411293ea41fef3ac8fa93f9de9e85f66bb"
                .to_string(),
            package_path: Some("docs/fixtures/rhist/real-ri-tract-unchanged".to_string()),
            cycle_ids: vec![
                "ri-2000-census".to_string(),
                "ri-2010-census".to_string(),
                "ri-2020-census".to_string(),
            ],
            role: "unit-lineage".to_string(),
            note: Some("Real-source RHIST pressure fixture.".to_string()),
        }];
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();

        write_package_dir(tmp.path(), &manifest, &package).unwrap();
        assert!(tmp.path().join("normalized/rhist-refs.ndjson").is_file());

        let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
        assert_eq!(decoded_package.rhist_refs.len(), 1);
        assert_eq!(
            decoded_package.rhist_refs[0].package_hash,
            "sha256:ccbddf423aa4ac08b0d45c4ac0b9db411293ea41fef3ac8fa93f9de9e85f66bb"
        );
        verify_package(&decoded_package).unwrap();
    }

    #[test]
    fn round_trips_rctx_references() {
        let tmp = tempfile::tempdir().unwrap();
        let mut package = synthetic_summary_basic_package();
        package.rctx_refs = vec![RctxReference {
            reference_id: "rctx:summary-basic-context".to_string(),
            context_hash: "sha256:1111111111111111111111111111111111111111111111111111111111111111"
                .to_string(),
            context_path: Some("context.rctx".to_string()),
            crosswalk_hash: Some(
                "sha256:2222222222222222222222222222222222222222222222222222222222222222"
                    .to_string(),
            ),
            crosswalk_path: Some("crosswalks/summary-basic-to-plan.ndjson".to_string()),
            role: "aggregation-crosswalk".to_string(),
            note: Some("Synthetic RCTX aggregation binding.".to_string()),
        }];
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();

        write_package_dir(tmp.path(), &manifest, &package).unwrap();
        assert!(tmp.path().join("normalized/rctx-refs.ndjson").is_file());

        let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
        assert_eq!(decoded_package.rctx_refs.len(), 1);
        assert_eq!(
            decoded_package.rctx_refs[0].crosswalk_hash.as_deref(),
            Some("sha256:2222222222222222222222222222222222222222222222222222222222222222")
        );
        verify_package(&decoded_package).unwrap();
    }

    #[test]
    fn round_trips_shared_rctx_rhist_base_references() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_summary_basic_package_with_base_references();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();

        write_package_dir(tmp.path(), &manifest, &package).unwrap();
        assert!(tmp.path().join("normalized/rctx-refs.ndjson").is_file());
        assert!(tmp.path().join("normalized/rhist-refs.ndjson").is_file());

        let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
        assert_eq!(
            decoded_package.rctx_refs[0].crosswalk_hash.as_deref(),
            Some(SYN_RCTX_L0_CROSSWALK_HASH)
        );
        assert_eq!(
            decoded_package.rhist_refs[0].package_hash,
            SYN_RHIST_L2_PACKAGE_HASH
        );
        verify_package(&decoded_package).unwrap();
    }

    #[test]
    fn round_trips_synthetic_bad_lineage_package() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_bad_lineage_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();
        let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
        assert_eq!(decoded_package.lineage.len(), 2);
        assert!(decoded_package.lineage[0]
            .current_reporting_unit_ids
            .contains(&"syn:precinct:P-004C".to_string()));
    }

    #[test]
    fn round_trips_synthetic_privacy_inclusion_package() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_privacy_inclusion_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();
        let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
        assert_eq!(decoded_package.inclusion_proofs.len(), 1);
        assert!(decoded_package.inclusion_proofs[0]
            .candidate_selections
            .is_empty());
    }

    #[test]
    fn round_trips_synthetic_choice_bearing_proof_package() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_choice_bearing_proof_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();
        let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
        assert_eq!(
            decoded_package.inclusion_proofs[0].candidate_selections,
            vec!["cand-a".to_string()]
        );
    }

    #[test]
    fn round_trips_synthetic_cvr_summary_package() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_cvr_summary_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();
        let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
        assert_eq!(decoded_package.cvr.len(), 140);
        assert!(decoded_package
            .cvr
            .iter()
            .any(|row| row.cvr_id == "cvr:P-001:001"));
    }

    #[test]
    fn round_trips_synthetic_bad_cvr_summary_package() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_bad_cvr_summary_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();
        let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
        assert_eq!(decoded_package.cvr.len(), 140);
        assert!(verify_source_index(tmp.path()).is_ok());
    }

    #[test]
    fn round_trips_synthetic_rla_replay_package() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_rla_replay_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();
        let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
        assert_eq!(decoded_package.rla_audits.len(), 1);
        assert_eq!(decoded_package.rla_audits[0].sample_draws.len(), 12);
    }

    #[test]
    fn round_trips_synthetic_bad_rla_replay_package() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_bad_rla_replay_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();
        let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
        assert_eq!(
            decoded_package.rla_audits[0].sample_draws[0].cvr_id,
            "cvr:P-999:999"
        );
    }

    #[test]
    fn round_trips_synthetic_rla_stopping_package() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_rla_stopping_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();
        let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
        assert_eq!(decoded_package.rla_audits[0].observations.len(), 12);
        assert_eq!(
            decoded_package.rla_audits[0].stopping_rule_id.as_deref(),
            Some("zero-discrepancy-threshold-v1")
        );
    }

    #[test]
    fn round_trips_synthetic_bad_rla_stopping_package() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_bad_rla_stopping_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();
        let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
        assert_eq!(
            decoded_package.rla_audits[0].observations[0].observed_selection_ids,
            vec!["cand-b".to_string()]
        );
    }

    #[test]
    fn round_trips_synthetic_rla_discrepancy_package() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_rla_discrepancy_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();
        let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
        assert_eq!(decoded_package.rla_audits[0].discrepancies.len(), 1);
    }

    #[test]
    fn round_trips_synthetic_bad_rla_discrepancy_package() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_bad_rla_discrepancy_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();
        let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
        assert_eq!(decoded_package.rla_audits[0].discrepancies.len(), 1);
    }

    #[test]
    fn round_trips_synthetic_rla_margin_package() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_rla_margin_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();
        let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
        assert_eq!(
            decoded_package.rla_audits[0]
                .margin
                .as_ref()
                .unwrap()
                .reported_margin,
            64
        );
    }

    #[test]
    fn round_trips_synthetic_bad_rla_margin_package() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_bad_rla_margin_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();
        let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
        assert_eq!(
            decoded_package.rla_audits[0]
                .margin
                .as_ref()
                .unwrap()
                .reported_margin,
            65
        );
    }

    #[test]
    fn round_trips_synthetic_rla_statistical_package() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_rla_statistical_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();
        let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
        assert_eq!(decoded_package.rla_audits[0].declared_risk_ppm, Some(1303));
    }

    #[test]
    fn round_trips_synthetic_bad_rla_statistical_package() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_bad_rla_statistical_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();
        let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
        assert_eq!(decoded_package.rla_audits[0].declared_risk_ppm, Some(1304));
    }

    #[test]
    fn round_trips_synthetic_colorado_rla_package() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_colorado_rla_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();
        let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
        assert_eq!(
            decoded_package.rla_audits[0]
                .jurisdiction_method_id
                .as_deref(),
            Some("colorado-rule-25-comparison-v1")
        );
    }

    #[test]
    fn round_trips_synthetic_bad_colorado_rla_package() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_bad_colorado_rla_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();
        let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
        assert_eq!(
            decoded_package.rla_audits[0].public_seed,
            "3141592653589793238X"
        );
    }

    #[test]
    fn round_trips_synthetic_california_rla_package() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_california_rla_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();
        let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
        assert_eq!(
            decoded_package.rla_audits[0]
                .jurisdiction_method_id
                .as_deref(),
            Some("california-public-rla-v1")
        );
    }

    #[test]
    fn round_trips_synthetic_bad_california_rla_package() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_bad_california_rla_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();
        let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
        assert_eq!(
            decoded_package.rla_audits[0]
                .audit_software_source_url
                .as_deref(),
            Some("synthetic-election-audit/rcount-open-rla-synthetic-v1")
        );
    }

    #[test]
    fn round_trips_synthetic_manual_audit_package() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_manual_audit_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();
        let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
        assert_eq!(decoded_package.manual_audits.len(), 1);
        assert_eq!(decoded_package.manual_audits[0].tolerance_votes, 0);
    }

    #[test]
    fn round_trips_synthetic_bad_manual_audit_package() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_bad_manual_audit_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();
        let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
        assert_eq!(decoded_package.manual_audits[0].hand_totals[1].votes, 36);
    }

    #[test]
    fn round_trips_synthetic_batch_comparison_package() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_batch_comparison_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();
        let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
        assert_eq!(decoded_package.batch_comparison_audits.len(), 1);
        assert_eq!(
            decoded_package.batch_comparison_audits[0].declared_overstatement,
            2
        );
        assert!(tmp.path().join("audits/batch-comparison.ndjson").exists());
    }

    #[test]
    fn round_trips_synthetic_batch_comparison_algorithm_package() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_batch_comparison_algorithm_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();
        let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
        let package_hashes: PackageHashes =
            read_json(&tmp.path().join("proofs/package-hashes.json")).unwrap();

        assert_eq!(package_hashes.audit_algorithm_run_count, 1);
        assert_eq!(package_hashes.batch_comparison_audit_count, 1);
        assert_eq!(decoded_package.audit_algorithm_runs.len(), 1);
        assert_eq!(decoded_package.batch_comparison_audits.len(), 1);
        assert_eq!(
            decoded_package.audit_algorithm_runs[0].sample_steps[0].sample_unit_id,
            "batch:P-001:election-day"
        );
        assert_eq!(
            decoded_package.audit_algorithm_runs[0].sample_steps[0]
                .assorter_value
                .denominator,
            5
        );
        assert!(tmp.path().join("audits/algorithm-runs.ndjson").exists());
        assert!(tmp.path().join("audits/batch-comparison.ndjson").exists());
    }

    #[test]
    fn round_trips_synthetic_kaplan_markov_macro_package() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_kaplan_markov_macro_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();
        let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
        let run = &decoded_package.audit_algorithm_runs[0];

        assert_eq!(run.macro_ballot_count, Some(100));
        assert_eq!(run.macro_reported_margin, Some(10));
        assert_eq!(
            run.macro_gamma,
            Some(rcount_core::RationalValue {
                numerator: 11,
                denominator: 10,
            })
        );
        assert_eq!(run.sample_steps.len(), 16);
        assert!(tmp.path().join("audits/algorithm-runs.ndjson").exists());
    }

    #[test]
    fn round_trips_synthetic_missing_hand_tally_batch_comparison_package() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_missing_hand_tally_batch_comparison_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();
        let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
        assert_eq!(
            decoded_package.batch_comparison_audits[0].hand_totals.len(),
            1
        );
    }

    #[test]
    fn round_trips_synthetic_batch_size_drift_comparison_package() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_batch_size_drift_comparison_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();
        let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
        assert_eq!(
            decoded_package.batch_comparison_audits[0].declared_batch_ballots,
            69
        );
    }

    #[test]
    fn rejects_manifest_content_hash_mismatch() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_summary_basic_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();
        let manifest_path = tmp.path().join("manifest.json");
        let mut raw: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&manifest_path).unwrap()).unwrap();
        raw["content_hash"] = serde_json::Value::String("sha256:bad".to_string());
        std::fs::write(&manifest_path, serde_json::to_vec_pretty(&raw).unwrap()).unwrap();
        assert!(matches!(
            read_package_dir(tmp.path()),
            Err(RcountIoError::ContentHashMismatch { .. })
        ));
    }

    #[test]
    fn round_trips_synthetic_bad_selection_sum_package() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_bad_selection_sum_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();
        let (_, decoded_package) = read_package_dir(tmp.path()).unwrap();
        assert_eq!(
            decoded_package.summaries[0].counted_ballots,
            synthetic_summary_basic_package().summaries[0].counted_ballots + 1
        );
        assert!(verify_source_index(tmp.path()).is_ok());
    }

    #[test]
    fn rejects_tampered_source_file() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_summary_basic_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();
        std::fs::write(
            tmp.path()
                .join("sources")
                .join("synthetic-summary-export.json"),
            br#"{"tampered":true}"#,
        )
        .unwrap();

        assert!(matches!(
            verify_source_index(tmp.path()),
            Err(RcountIoError::SourceHashMismatch { .. })
        ));
    }

    #[test]
    fn rejects_empty_source_index() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_summary_basic_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();
        std::fs::write(
            tmp.path().join("sources").join("source-index.json"),
            br#"{"sources":[]}"#,
        )
        .unwrap();

        assert!(matches!(
            verify_source_index(tmp.path()),
            Err(RcountIoError::EmptySourceIndex)
        ));
    }

    #[test]
    fn docs_summary_basic_fixture_verifies_when_present() {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("docs")
            .join("examples")
            .join("rcount-golden-packages")
            .join("summary-basic");
        if dir.exists() {
            verify_summary_basic_dir(&dir).unwrap();
        }
    }
}
