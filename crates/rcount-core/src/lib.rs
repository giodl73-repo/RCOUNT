use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

pub const RCOUNT_VERSION: &str = "0.1-draft";
pub const SOURCE_HASH_PREFIX: &[u8] = b"RCOUNT_SOURCE_V1\0";
pub const RECORD_HASH_PREFIX: &[u8] = b"RCOUNT_RECORD_V1\0";
pub const FILE_HASH_PREFIX: &[u8] = b"RCOUNT_FILE_V1\0";
pub const PACKAGE_HASH_PREFIX: &[u8] = b"RCOUNT_PACKAGE_V1\0";
pub const EVENT_HASH_PREFIX: &[u8] = b"RCOUNT_EVENT_V1\0";
pub const PROOF_HASH_PREFIX: &[u8] = b"RCOUNT_PROOF_V1\0";
pub const RLA_MANIFEST_HASH_PREFIX: &[u8] = b"RCOUNT_RLA_MANIFEST_V1\0";
pub const RLA_SAMPLE_PREFIX: &[u8] = b"RCOUNT_RLA_SAMPLE_V1\0";
pub const RLA_SAMPLING_ALGORITHM_ID: &str = "rcount-sha256-modulo-v1";
pub const COLORADO_RLA_METHOD_ID: &str = "colorado-rule-25-comparison-v1";
pub const CALIFORNIA_RLA_METHOD_ID: &str = "california-public-rla-v1";
pub const CALIFORNIA_BALLOT_MANIFEST_FORMAT_ID: &str =
    "ca-post-election-rla-ballot-manifest-2019-10-15";
pub const BRAVO_BALLOT_POLLING_METHOD_ID: &str = "bravo-ballot-polling-v1";
pub const MINERVA_BALLOT_POLLING_METHOD_ID: &str = "minerva-ballot-polling-v1";
pub const ATHENA_BALLOT_POLLING_METHOD_ID: &str = "athena-ballot-polling-v1";
pub const KAPLAN_MARKOV_COMPARISON_METHOD_ID: &str = "kaplan-markov-comparison-v1";
pub const ALPHA_MARTINGALE_METHOD_ID: &str = "alpha-martingale-v1";
pub const SHANGRLA_ASSORTER_METHOD_ID: &str = "shangrla-assorter-v1";
pub const STRATIFIED_HYBRID_RLA_METHOD_ID: &str = "stratified-hybrid-rla-v1";
pub const BATCH_COMPARISON_METHOD_ID: &str = "batch-comparison-v1";
pub const RAIRE_IRV_METHOD_ID: &str = "raire-irv-v1";
pub const AWAIRE_IRV_METHOD_ID: &str = "awaire-irv-v1";
pub const BAYESIAN_TABULATION_AUDIT_METHOD_ID: &str = "bayesian-tabulation-audit-v1";
pub const SOBA_OBSERVABLE_BALLOT_AUDIT_METHOD_ID: &str = "soba-observable-ballot-audit-v1";
pub const SYN_RCTX_L0_PACKAGE_HASH: &str =
    "sha256:bf552e9d9753d3376155ca9c4b21db6b1930e37919a58bcb9096cd563653d532";
pub const SYN_RCTX_L0_CONTEXT_HASH: &str =
    "sha256:b11f1eabcaf33e2d2691ddbe498c650830cffb9b0fb62820292d4ca0166c0bb7";
pub const SYN_RCTX_L0_CROSSWALK_HASH: &str =
    "sha256:906054d087e8c006047448c821d79e75a81bb1bbeb1d9349ab7b5d025029d9bb";
pub const SYN_RHIST_L2_PACKAGE_HASH: &str =
    "sha256:2c391099d7b61ba0c27fd231376391aadec81de62a387e291a043ed18d69db0b";

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum RcountCoreError {
    #[error("canonical JSON error: {0}")]
    CanonicalJson(String),
    #[error("duplicate selection id in contest {contest_id}: {selection_id}")]
    DuplicateSelectionId {
        contest_id: String,
        selection_id: String,
    },
    #[error("summary for contest {contest_id} reporting unit {reporting_unit_id} has duplicate selection id: {selection_id}")]
    DuplicateSummarySelection {
        contest_id: String,
        reporting_unit_id: String,
        selection_id: String,
    },
    #[error("summary for contest {contest_id} reporting unit {reporting_unit_id} references unknown selection id: {selection_id}")]
    UnknownSelection {
        contest_id: String,
        reporting_unit_id: String,
        selection_id: String,
    },
    #[error("votes and residual counts must be non-negative")]
    NegativeCount,
    #[error("contest selection sum mismatch for contest {contest_id} reporting unit {reporting_unit_id}: declared {declared_ballots}, computed {computed_ballots}")]
    ContestSelectionSumMismatch {
        contest_id: String,
        reporting_unit_id: String,
        declared_ballots: i64,
        computed_ballots: i64,
    },
    #[error("missing jurisdiction total summary for contest {contest_id} reporting unit {jurisdiction_reporting_unit_id}")]
    MissingJurisdictionTotal {
        contest_id: String,
        jurisdiction_reporting_unit_id: String,
    },
    #[error("jurisdiction total mismatch for contest {contest_id} selection {selection_id}: declared {declared_votes}, computed {computed_votes}")]
    JurisdictionSelectionMismatch {
        contest_id: String,
        selection_id: String,
        declared_votes: i64,
        computed_votes: i64,
    },
    #[error("jurisdiction residual mismatch for contest {contest_id} field {field}: declared {declared}, computed {computed}")]
    JurisdictionResidualMismatch {
        contest_id: String,
        field: String,
        declared: i64,
        computed: i64,
    },
    #[error("duplicate status event id: {event_id}")]
    DuplicateStatusEventId { event_id: String },
    #[error("status event {event_id} has the same before and after status")]
    NoStatusTransition { event_id: String },
    #[error("status event {event_id} must include authority and explanation")]
    IncompleteStatusEvent { event_id: String },
    #[error("missing canvass correction event from unofficial to canvassed")]
    MissingCanvassCorrectionEvent,
    #[error("missing summaries for status {status:?}")]
    MissingStatusSummaries { status: CountStatus },
    #[error("duplicate batch id: {batch_id}")]
    DuplicateBatchId { batch_id: String },
    #[error("summary for contest {contest_id} reporting unit {reporting_unit_id} references missing batch id: {batch_id}")]
    MissingBatch {
        contest_id: String,
        reporting_unit_id: String,
        batch_id: String,
    },
    #[error("batch total mismatch for batch {batch_id}: declared {declared_ballots}, summary {summary_ballots}")]
    BatchSummaryTotalMismatch {
        batch_id: String,
        declared_ballots: i64,
        summary_ballots: i64,
    },
    #[error("accepted ballots mismatch for batch {batch_id}: declared {declared_ballots}, counted plus rejected {computed_ballots}")]
    AcceptedBallotsMismatch {
        batch_id: String,
        declared_ballots: i64,
        computed_ballots: i64,
    },
    #[error("duplicate lineage event id: {lineage_id}")]
    DuplicateLineageId { lineage_id: String },
    #[error(
        "lineage event {lineage_id} references missing prior reporting unit: {reporting_unit_id}"
    )]
    MissingPriorLineageUnit {
        lineage_id: String,
        reporting_unit_id: String,
    },
    #[error(
        "lineage event {lineage_id} references missing current reporting unit: {reporting_unit_id}"
    )]
    MissingCurrentLineageUnit {
        lineage_id: String,
        reporting_unit_id: String,
    },
    #[error("lineage event {lineage_id} has invalid split cardinality")]
    InvalidSplitLineage { lineage_id: String },
    #[error("lineage event {lineage_id} has invalid merge cardinality")]
    InvalidMergeLineage { lineage_id: String },
    #[error("duplicate RHIST reference id: {reference_id}")]
    DuplicateRhistReference { reference_id: String },
    #[error("RHIST reference {reference_id} has invalid package hash: {package_hash}")]
    InvalidRhistPackageHash {
        reference_id: String,
        package_hash: String,
    },
    #[error("RHIST reference {reference_id} must include at least one cycle id")]
    EmptyRhistCycleRefs { reference_id: String },
    #[error("RHIST reference {reference_id} has unsupported role: {role}")]
    UnsupportedRhistReferenceRole { reference_id: String, role: String },
    #[error("duplicate RCTX reference id: {reference_id}")]
    DuplicateRctxReference { reference_id: String },
    #[error("RCTX reference {reference_id} has invalid context hash: {context_hash}")]
    InvalidRctxContextHash {
        reference_id: String,
        context_hash: String,
    },
    #[error("RCTX reference {reference_id} has invalid crosswalk hash: {crosswalk_hash}")]
    InvalidRctxCrosswalkHash {
        reference_id: String,
        crosswalk_hash: String,
    },
    #[error("RCTX reference {reference_id} has unsupported role: {role}")]
    UnsupportedRctxReferenceRole { reference_id: String, role: String },
    #[error("duplicate proof id: {proof_id}")]
    DuplicateProofId { proof_id: String },
    #[error("proof {proof_id} exposes candidate selections")]
    ChoiceBearingProof { proof_id: String },
    #[error("proof {proof_id} combines voter identity with ballot style and timestamp")]
    LinkableVoterProof { proof_id: String },
    #[error("proof {proof_id} has invalid token hash: {token_hash}")]
    InvalidProofTokenHash {
        proof_id: String,
        token_hash: String,
    },
    #[error("duplicate CVR contest row for cvr {cvr_id} contest {contest_id}")]
    DuplicateCvrContest { cvr_id: String, contest_id: String },
    #[error("CVR contest row {cvr_id}/{contest_id} has invalid mark cardinality")]
    InvalidCvrContestCardinality { cvr_id: String, contest_id: String },
    #[error(
        "CVR contest row {cvr_id}/{contest_id} references unknown selection id: {selection_id}"
    )]
    UnknownCvrSelection {
        cvr_id: String,
        contest_id: String,
        selection_id: String,
    },
    #[error(
        "missing summary for CVR aggregate contest {contest_id} reporting unit {reporting_unit_id}"
    )]
    MissingCvrSummary {
        contest_id: String,
        reporting_unit_id: String,
    },
    #[error("CVR summary mismatch for contest {contest_id} reporting unit {reporting_unit_id} field {field}: summary {summary}, cvr {cvr}")]
    CvrSummaryMismatch {
        contest_id: String,
        reporting_unit_id: String,
        field: String,
        summary: i64,
        cvr: i64,
    },
    #[error("duplicate RLA audit id: {audit_id}")]
    DuplicateRlaAuditId { audit_id: String },
    #[error("RLA audit {audit_id} has invalid risk limit ppm: {risk_limit_ppm}")]
    InvalidRlaRiskLimit {
        audit_id: String,
        risk_limit_ppm: u32,
    },
    #[error("RLA audit {audit_id} has invalid sample size: {sample_size}")]
    InvalidRlaSampleSize { audit_id: String, sample_size: u32 },
    #[error("RLA audit {audit_id} has unsupported sampling algorithm: {sampling_algorithm_id}")]
    UnsupportedRlaSamplingAlgorithm {
        audit_id: String,
        sampling_algorithm_id: String,
    },
    #[error("duplicate audit algorithm run id: {run_id}")]
    DuplicateAuditAlgorithmRunId { run_id: String },
    #[error("audit algorithm run {run_id} has invalid risk limit ppm: {risk_limit_ppm}")]
    InvalidAuditAlgorithmRiskLimit { run_id: String, risk_limit_ppm: u32 },
    #[error("audit algorithm run {run_id} has invalid MACRO design fields")]
    InvalidAuditMacroDesign { run_id: String },
    #[error("audit algorithm run {run_id} has invalid stratified/hybrid design")]
    InvalidStratifiedHybridDesign { run_id: String },
    #[error("audit algorithm run {run_id} references missing stratified/hybrid component run: {component_run_id}")]
    MissingStratifiedHybridComponent {
        run_id: String,
        component_run_id: String,
    },
    #[error("audit algorithm run {run_id} has invalid ranked-choice audit design")]
    InvalidRankedChoiceAuditDesign { run_id: String },
    #[error("audit algorithm run {run_id} step {step_index} has invalid ranked choices")]
    InvalidRankedChoiceSample { run_id: String, step_index: u32 },
    #[error("audit algorithm run {run_id} has invalid Bayesian audit design")]
    InvalidBayesianAuditDesign { run_id: String },
    #[error("audit algorithm run {run_id} has invalid observable-ballot audit design")]
    InvalidObservableBallotAuditDesign { run_id: String },
    #[error("audit algorithm run {run_id} step {step_index} references missing observable-ballot opening: {proof_id}")]
    MissingObservableBallotOpening {
        run_id: String,
        step_index: u32,
        proof_id: String,
    },
    #[error("audit algorithm run {run_id} has unsupported method id: {method_id}")]
    UnsupportedAuditAlgorithmMethod { run_id: String, method_id: String },
    #[error("audit algorithm run {run_id} has duplicate assertion id: {assertion_id}")]
    DuplicateAuditAssertion {
        run_id: String,
        assertion_id: String,
    },
    #[error("audit algorithm run {run_id} assertion {assertion_id} has invalid assorter bound")]
    InvalidAuditAssorterBound {
        run_id: String,
        assertion_id: String,
    },
    #[error("audit algorithm run {run_id} step {step_index} references missing assertion {assertion_id}")]
    MissingAuditAssertion {
        run_id: String,
        step_index: u32,
        assertion_id: String,
    },
    #[error("audit algorithm run {run_id} has duplicate sample step {step_index} for assertion {assertion_id}")]
    DuplicateAuditSampleStep {
        run_id: String,
        assertion_id: String,
        step_index: u32,
    },
    #[error("audit algorithm run {run_id} step {step_index} has invalid assorter value")]
    InvalidAuditAssorterValue { run_id: String, step_index: u32 },
    #[error(
        "audit algorithm run {run_id} step {step_index} has invalid p-value ppm: {p_value_ppm}"
    )]
    InvalidAuditPValue {
        run_id: String,
        step_index: u32,
        p_value_ppm: u32,
    },
    #[error("audit algorithm run {run_id} step {step_index} references missing batch comparison audit for batch {batch_id}")]
    MissingBatchComparisonAlgorithmEvidence {
        run_id: String,
        step_index: u32,
        batch_id: String,
    },
    #[error("audit algorithm run {run_id} step {step_index} batch comparison taint mismatch: declared {declared:?}, computed {computed:?}")]
    BatchComparisonAlgorithmTaintMismatch {
        run_id: String,
        step_index: u32,
        declared: RationalValue,
        computed: RationalValue,
    },
    #[error("audit algorithm run {run_id} batch comparison sample order is empty")]
    EmptyBatchComparisonAlgorithmSample { run_id: String },
    #[error("audit algorithm run {run_id} batch comparison audit {audit_id} has nonpositive reported margin: {reported_margin}")]
    InvalidBatchComparisonAlgorithmMargin {
        run_id: String,
        audit_id: String,
        reported_margin: i64,
    },
    #[error("audit algorithm run {run_id} batch comparison audit {audit_id} assertion mismatch")]
    BatchComparisonAlgorithmAssertionMismatch { run_id: String, audit_id: String },
    #[error("RLA audit {audit_id} has no CVR population for contest {contest_id}")]
    MissingRlaPopulation {
        audit_id: String,
        contest_id: String,
    },
    #[error(
        "RLA audit {audit_id} manifest hash mismatch: declared {declared}, computed {computed}"
    )]
    RlaManifestHashMismatch {
        audit_id: String,
        declared: String,
        computed: String,
    },
    #[error("RLA audit {audit_id} sample mismatch at draw {draw_index}: declared {declared_cvr_id}, computed {computed_cvr_id}")]
    RlaSampleMismatch {
        audit_id: String,
        draw_index: u32,
        declared_cvr_id: String,
        computed_cvr_id: String,
    },
    #[error("RLA audit {audit_id} has incomplete stopping-rule fields")]
    MissingRlaStoppingRule { audit_id: String },
    #[error("RLA audit {audit_id} has duplicate observation for draw {draw_index}")]
    DuplicateRlaObservation { audit_id: String, draw_index: u32 },
    #[error("RLA audit {audit_id} is missing observation for draw {draw_index}")]
    MissingRlaObservation { audit_id: String, draw_index: u32 },
    #[error("RLA audit {audit_id} observation draw {draw_index} references cvr {observed_cvr_id}, expected {expected_cvr_id}")]
    RlaObservationCvrMismatch {
        audit_id: String,
        draw_index: u32,
        expected_cvr_id: String,
        observed_cvr_id: String,
    },
    #[error("RLA audit {audit_id} declares status {declared:?}, computed {computed:?}")]
    RlaStoppingStatusMismatch {
        audit_id: String,
        declared: RlaStoppingStatus,
        computed: RlaStoppingStatus,
    },
    #[error("RLA audit {audit_id} declared discrepancy count {declared}, computed {computed}")]
    RlaDiscrepancyCountMismatch {
        audit_id: String,
        declared: usize,
        computed: usize,
    },
    #[error("RLA audit {audit_id} discrepancy mismatch at draw {draw_index}: declared {declared:?}, computed {computed:?}")]
    RlaDiscrepancyMismatch {
        audit_id: String,
        draw_index: u32,
        declared: RlaDiscrepancyKind,
        computed: RlaDiscrepancyKind,
    },
    #[error("RLA audit {audit_id} is missing margin metadata")]
    MissingRlaMarginMetadata { audit_id: String },
    #[error(
        "RLA audit {audit_id} margin metadata references missing selection id: {selection_id}"
    )]
    MissingRlaMarginSelection {
        audit_id: String,
        selection_id: String,
    },
    #[error("RLA audit {audit_id} reported margin is not positive: {margin}")]
    InvalidRlaReportedMargin { audit_id: String, margin: i64 },
    #[error("RLA audit {audit_id} reported winner votes mismatch for {selection_id}: declared {declared}, summary {summary}")]
    RlaWinnerVotesMismatch {
        audit_id: String,
        selection_id: String,
        declared: i64,
        summary: i64,
    },
    #[error("RLA audit {audit_id} reported loser votes mismatch for {selection_id}: declared {declared}, summary {summary}")]
    RlaLoserVotesMismatch {
        audit_id: String,
        selection_id: String,
        declared: i64,
        summary: i64,
    },
    #[error(
        "RLA audit {audit_id} reported margin mismatch: declared {declared}, summary {summary}"
    )]
    RlaReportedMarginMismatch {
        audit_id: String,
        declared: i64,
        summary: i64,
    },
    #[error("RLA audit {audit_id} diluted margin denominator mismatch: declared {declared}, summary {summary}")]
    RlaDilutedMarginDenominatorMismatch {
        audit_id: String,
        declared: i64,
        summary: i64,
    },
    #[error("RLA audit {audit_id} is missing statistical risk estimate")]
    MissingRlaRiskEstimate { audit_id: String },
    #[error("RLA audit {audit_id} risk estimate mismatch: declared {declared_ppm} ppm, computed {computed_ppm} ppm")]
    RlaRiskEstimateMismatch {
        audit_id: String,
        declared_ppm: u32,
        computed_ppm: u32,
    },
    #[error("RLA audit {audit_id} has unsupported jurisdiction method: {jurisdiction_method_id}")]
    UnsupportedRlaJurisdictionMethod {
        audit_id: String,
        jurisdiction_method_id: String,
    },
    #[error("RLA audit {audit_id} has invalid Colorado-style public seed: {public_seed}")]
    InvalidColoradoRlaSeed {
        audit_id: String,
        public_seed: String,
    },
    #[error("RLA audit {audit_id} is missing Colorado-style comparison audit fields")]
    MissingColoradoRlaComparisonFields { audit_id: String },
    #[error("RLA audit {audit_id} is missing California-style public audit tool fields")]
    MissingCaliforniaRlaPublicToolFields { audit_id: String },
    #[error("RLA audit {audit_id} has invalid California-style ballot manifest format: {ballot_manifest_format_id}")]
    InvalidCaliforniaRlaManifestFormat {
        audit_id: String,
        ballot_manifest_format_id: String,
    },
    #[error("RLA audit {audit_id} has invalid public audit software source URL: {source_url}")]
    InvalidRlaSoftwareSourceUrl {
        audit_id: String,
        source_url: String,
    },
    #[error("duplicate manual audit id: {audit_id}")]
    DuplicateManualAuditId { audit_id: String },
    #[error("manual audit {audit_id} is missing canvassed summary for contest {contest_id} reporting unit {reporting_unit_id}")]
    MissingManualAuditSummary {
        audit_id: String,
        contest_id: String,
        reporting_unit_id: String,
    },
    #[error("manual audit {audit_id} machine total mismatch for {selection_id}: declared {declared}, summary {summary}")]
    ManualAuditMachineTotalMismatch {
        audit_id: String,
        selection_id: String,
        declared: i64,
        summary: i64,
    },
    #[error("manual audit {audit_id} declares status {declared:?}, computed {computed:?}")]
    ManualAuditStatusMismatch {
        audit_id: String,
        declared: ManualAuditStatus,
        computed: ManualAuditStatus,
    },
    #[error("duplicate batch comparison audit id: {audit_id}")]
    DuplicateBatchComparisonAuditId { audit_id: String },
    #[error("batch comparison audit {audit_id} references missing batch: {batch_id}")]
    MissingBatchComparisonBatch { audit_id: String, batch_id: String },
    #[error("batch comparison audit {audit_id} batch size mismatch for {batch_id}: declared {declared}, batch manifest {manifest}")]
    BatchComparisonBatchSizeMismatch {
        audit_id: String,
        batch_id: String,
        declared: i64,
        manifest: i64,
    },
    #[error("batch comparison audit {audit_id} is missing batch summary for contest {contest_id} batch {batch_id}")]
    MissingBatchComparisonSummary {
        audit_id: String,
        contest_id: String,
        batch_id: String,
    },
    #[error("batch comparison audit {audit_id} reported total mismatch for {selection_id}: declared {declared}, summary {summary}")]
    BatchComparisonReportedTotalMismatch {
        audit_id: String,
        selection_id: String,
        declared: i64,
        summary: i64,
    },
    #[error("batch comparison audit {audit_id} is missing hand tally for {selection_id}")]
    MissingBatchComparisonHandTally {
        audit_id: String,
        selection_id: String,
    },
    #[error("batch comparison audit {audit_id} declared reported margin {declared}, computed {computed}")]
    BatchComparisonReportedMarginMismatch {
        audit_id: String,
        declared: i64,
        computed: i64,
    },
    #[error(
        "batch comparison audit {audit_id} declared hand margin {declared}, computed {computed}"
    )]
    BatchComparisonHandMarginMismatch {
        audit_id: String,
        declared: i64,
        computed: i64,
    },
    #[error(
        "batch comparison audit {audit_id} declared overstatement {declared}, computed {computed}"
    )]
    BatchComparisonOverstatementMismatch {
        audit_id: String,
        declared: i64,
        computed: i64,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SelectionKind {
    Candidate,
    WriteInBucket,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Selection {
    pub selection_id: String,
    pub kind: SelectionKind,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Contest {
    pub contest_id: String,
    pub title: String,
    pub vote_for: u32,
    pub selections: Vec<Selection>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ReportingUnitKind {
    Precinct,
    SplitPrecinct,
    VoteCenter,
    CentralCountBatch,
    MailBatch,
    ProvisionalBatch,
    JurisdictionTotal,
    DistrictTotal,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReportingUnit {
    pub reporting_unit_id: String,
    pub kind: ReportingUnitKind,
    pub parent_jurisdiction: String,
    #[serde(default)]
    pub source_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_from: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_to: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SelectionTotal {
    pub selection_id: String,
    pub votes: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Summary {
    pub contest_id: String,
    pub reporting_unit_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub batch_id: Option<String>,
    pub status: CountStatus,
    pub totals: Vec<SelectionTotal>,
    pub undervotes: i64,
    pub overvotes: i64,
    pub blank_contests: i64,
    pub counted_ballots: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BatchKind {
    ElectionDay,
    Mail,
    Provisional,
    CentralCount,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchManifest {
    pub batch_id: String,
    pub reporting_unit_id: String,
    pub kind: BatchKind,
    pub status: CountStatus,
    pub accepted_ballots: i64,
    pub counted_ballots: i64,
    pub rejected_ballots: i64,
    #[serde(default)]
    pub source_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LineageKind {
    Unchanged,
    Split,
    Merge,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReportingUnitLineage {
    pub lineage_id: String,
    pub kind: LineageKind,
    pub prior_cycle: String,
    pub current_cycle: String,
    pub prior_reporting_unit_ids: Vec<String>,
    pub current_reporting_unit_ids: Vec<String>,
    pub authority: String,
    pub explanation: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RhistReference {
    pub reference_id: String,
    pub package_hash: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub package_path: Option<String>,
    #[serde(default)]
    pub cycle_ids: Vec<String>,
    pub role: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RctxReference {
    pub reference_id: String,
    pub context_hash: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub crosswalk_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub crosswalk_path: Option<String>,
    pub role: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum InclusionProofKind {
    AnonymizedAcceptedBallotToken,
    AnonymizedCountedBallotToken,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InclusionProof {
    pub proof_id: String,
    pub kind: InclusionProofKind,
    pub token_hash: String,
    pub reporting_unit_id: String,
    #[serde(default)]
    pub candidate_selections: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub voter_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ballot_style: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issued_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CvrContestRecord {
    pub cvr_id: String,
    pub contest_id: String,
    pub reporting_unit_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub batch_id: Option<String>,
    pub status: CountStatus,
    #[serde(default)]
    pub selection_ids: Vec<String>,
    #[serde(default)]
    pub undervote: bool,
    #[serde(default)]
    pub overvote: bool,
    #[serde(default)]
    pub blank_contest: bool,
    #[serde(default)]
    pub source_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RiskLimitAudit {
    pub audit_id: String,
    pub contest_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub jurisdiction_method_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ballot_manifest_format_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audit_software_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audit_software_source_url: Option<String>,
    pub risk_limit_ppm: u32,
    pub public_seed: String,
    pub sampling_algorithm_id: String,
    pub manifest_hash: String,
    pub sample_size: u32,
    pub sample_draws: Vec<RlaSampleDraw>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub observations: Vec<RlaSampleObservation>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub discrepancies: Vec<RlaDiscrepancy>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub margin: Option<RlaMarginMetadata>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stopping_rule_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_discrepancies: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub declared_status: Option<RlaStoppingStatus>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub declared_risk_ppm: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RlaSampleDraw {
    pub draw_index: u32,
    pub cvr_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RlaSampleObservation {
    pub draw_index: u32,
    pub cvr_id: String,
    #[serde(default)]
    pub observed_selection_ids: Vec<String>,
    #[serde(default)]
    pub undervote: bool,
    #[serde(default)]
    pub overvote: bool,
    #[serde(default)]
    pub blank_contest: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RlaDiscrepancyKind {
    SelectionMismatch,
    ResidualMismatch,
    SelectionAndResidualMismatch,
    WrongCvrObserved,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RlaDiscrepancy {
    pub draw_index: u32,
    pub cvr_id: String,
    pub kind: RlaDiscrepancyKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RlaMarginMetadata {
    pub winner_selection_id: String,
    pub loser_selection_id: String,
    pub reported_winner_votes: i64,
    pub reported_loser_votes: i64,
    pub reported_margin: i64,
    pub diluted_margin_denominator: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RationalValue {
    pub numerator: i64,
    pub denominator: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AuditAssertionKind {
    PluralityWinnerLoser,
    AssorterMean,
    ComparisonOverstatement,
    RankedChoiceAssertion,
    BayesianOutcome,
    ObservableBallotLinkage,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditAssertion {
    pub assertion_id: String,
    pub kind: AuditAssertionKind,
    pub assorter_id: String,
    pub assorter_upper_bound: RationalValue,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub winner_selection_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub loser_selection_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AuditSamplingMode {
    WithReplacement,
    WithoutReplacement,
    Bernoulli,
    Weighted,
    Batch,
    BoundaryOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AuditAlgorithmDecision {
    Pass,
    Continue,
    Escalate,
    Boundary,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditSampleStep {
    pub step_index: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub round_index: Option<u32>,
    pub assertion_id: String,
    pub sample_unit_id: String,
    pub assorter_value: RationalValue,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bet: Option<RationalValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub statistic: Option<RationalValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub p_value_ppm: Option<u32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ranked_choices: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditStratum {
    pub stratum_id: String,
    pub method_id: String,
    pub component_run_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ballot_count: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allocation_ppm: Option<u32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditAlgorithmRun {
    pub run_id: String,
    pub contest_id: String,
    pub method_id: String,
    pub sampling_mode: AuditSamplingMode,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rcv_elimination_order: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub risk_limit_ppm: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reported_winner_votes: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reported_loser_votes: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub macro_ballot_count: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub macro_reported_margin: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub macro_gamma: Option<RationalValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub combining_rule_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nuisance_parameter: Option<RationalValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bayesian_prior_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bayesian_likelihood_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub posterior_winner_probability_ppm: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub posterior_risk_ppm: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub simulation_seed: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub posterior_draws: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub calibrated_risk_limit_ppm: Option<u32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub strata: Vec<AuditStratum>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub assertions: Vec<AuditAssertion>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sample_steps: Vec<AuditSampleStep>,
    pub decision: AuditAlgorithmDecision,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_refs: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RlaStoppingStatus {
    Pass,
    Escalate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ManualAuditStatus {
    Pass,
    Escalate,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManualAudit {
    pub audit_id: String,
    pub contest_id: String,
    pub reporting_unit_id: String,
    pub authority: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub audited_batch_ids: Vec<String>,
    pub tolerance_votes: i64,
    pub machine_totals: Vec<SelectionTotal>,
    pub hand_totals: Vec<SelectionTotal>,
    pub declared_status: ManualAuditStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchComparisonAudit {
    pub audit_id: String,
    pub contest_id: String,
    pub batch_id: String,
    pub declared_batch_ballots: i64,
    pub winner_selection_id: String,
    pub loser_selection_id: String,
    pub reported_totals: Vec<SelectionTotal>,
    pub hand_totals: Vec<SelectionTotal>,
    pub declared_reported_margin: i64,
    pub declared_hand_margin: i64,
    pub declared_overstatement: i64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_refs: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CountStatus {
    Unofficial,
    Canvassed,
    Recounted,
    Amended,
    Certified,
    Withdrawn,
    Superseded,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum StatusEventType {
    InitialUnofficialReport,
    LateMailBatchAdded,
    ProvisionalAdjudication,
    BallotCureUpdate,
    DuplicateBallotResolution,
    WriteInAdjudication,
    RecountUpdate,
    CourtOrder,
    Certification,
    AmendedCertification,
    Correction,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatusEvent {
    pub event_id: String,
    pub event_type: StatusEventType,
    pub status_before: CountStatus,
    pub status_after: CountStatus,
    pub effective_at: String,
    pub authority: String,
    #[serde(default)]
    pub source_refs: Vec<String>,
    pub explanation: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RcountPackage {
    pub rcount_version: String,
    pub contests: Vec<Contest>,
    pub reporting_units: Vec<ReportingUnit>,
    #[serde(default)]
    pub batches: Vec<BatchManifest>,
    #[serde(default)]
    pub lineage: Vec<ReportingUnitLineage>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rhist_refs: Vec<RhistReference>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rctx_refs: Vec<RctxReference>,
    #[serde(default)]
    pub inclusion_proofs: Vec<InclusionProof>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cvr: Vec<CvrContestRecord>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub audit_algorithm_runs: Vec<AuditAlgorithmRun>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rla_audits: Vec<RiskLimitAudit>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub manual_audits: Vec<ManualAudit>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub batch_comparison_audits: Vec<BatchComparisonAudit>,
    pub summaries: Vec<Summary>,
    #[serde(default)]
    pub status_events: Vec<StatusEvent>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EquationPass {
    pub equation_id: String,
    pub contest_id: String,
    pub reporting_unit_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct VerificationReport {
    pub passed: Vec<EquationPass>,
    pub failed: Vec<String>,
}

pub fn canonical_hash(prefix: &[u8], value: &Value) -> Result<String, RcountCoreError> {
    let canonical = canonicalize_value(value);
    let bytes = serde_json::to_vec(&canonical)
        .map_err(|err| RcountCoreError::CanonicalJson(err.to_string()))?;
    let mut h = Sha256::new();
    h.update(prefix);
    h.update(bytes);
    Ok(format!("sha256:{:x}", h.finalize()))
}

pub fn record_hash<T: Serialize>(record: &T) -> Result<String, RcountCoreError> {
    let value = serde_json::to_value(record)
        .map_err(|err| RcountCoreError::CanonicalJson(err.to_string()))?;
    canonical_hash(RECORD_HASH_PREFIX, &value)
}

pub fn package_content_hash(package: &RcountPackage) -> Result<String, RcountCoreError> {
    let value = serde_json::to_value(package)
        .map_err(|err| RcountCoreError::CanonicalJson(err.to_string()))?;
    canonical_hash(PACKAGE_HASH_PREFIX, &value)
}

pub fn verify_package(package: &RcountPackage) -> Result<VerificationReport, RcountCoreError> {
    let contests: BTreeMap<&str, &Contest> = package
        .contests
        .iter()
        .map(|contest| (contest.contest_id.as_str(), contest))
        .collect();
    for contest in package.contests.iter() {
        validate_contest(contest)?;
    }

    let mut report = VerificationReport::default();
    for summary in package.summaries.iter() {
        let contest = contests.get(summary.contest_id.as_str()).ok_or_else(|| {
            RcountCoreError::UnknownSelection {
                contest_id: summary.contest_id.clone(),
                reporting_unit_id: summary.reporting_unit_id.clone(),
                selection_id: "<contest-missing>".to_string(),
            }
        })?;
        verify_contest_selection_sum(contest, summary)?;
        report.passed.push(EquationPass {
            equation_id: "contest_selection_sum".to_string(),
            contest_id: summary.contest_id.clone(),
            reporting_unit_id: summary.reporting_unit_id.clone(),
        });
    }
    report.passed.extend(verify_status_events(package)?);
    report.passed.extend(verify_batch_summary_totals(package)?);
    report.passed.extend(verify_lineage_conservation(package)?);
    report.passed.extend(verify_rhist_references(package)?);
    report.passed.extend(verify_rctx_references(package)?);
    report.passed.extend(verify_proof_privacy(package)?);
    report
        .passed
        .extend(verify_cvr_summary_reconciliation(package)?);
    report
        .passed
        .extend(verify_batch_comparison_audits(package)?);
    report.passed.extend(verify_audit_algorithm_runs(package)?);
    report.passed.extend(verify_rla_sampler_replay(package)?);
    report.passed.extend(verify_rla_margin_metadata(package)?);
    report.passed.extend(verify_rla_stopping_rules(package)?);
    report
        .passed
        .extend(verify_rla_jurisdiction_adapters(package)?);
    report.passed.extend(verify_manual_audits(package)?);
    Ok(report)
}

pub fn verify_package_parallel(
    package: &RcountPackage,
) -> Result<VerificationReport, RcountCoreError> {
    let contests: BTreeMap<&str, &Contest> = package
        .contests
        .iter()
        .map(|contest| (contest.contest_id.as_str(), contest))
        .collect();
    for contest in package.contests.iter() {
        validate_contest(contest)?;
    }

    let summary_passes = package
        .summaries
        .par_iter()
        .map(|summary| {
            let contest = contests.get(summary.contest_id.as_str()).ok_or_else(|| {
                RcountCoreError::UnknownSelection {
                    contest_id: summary.contest_id.clone(),
                    reporting_unit_id: summary.reporting_unit_id.clone(),
                    selection_id: "<contest-missing>".to_string(),
                }
            })?;
            verify_contest_selection_sum(contest, summary)?;
            Ok(EquationPass {
                equation_id: "contest_selection_sum".to_string(),
                contest_id: summary.contest_id.clone(),
                reporting_unit_id: summary.reporting_unit_id.clone(),
            })
        })
        .collect::<Result<Vec<_>, RcountCoreError>>()?;

    let mut report = VerificationReport::default();
    report.passed.extend(summary_passes);
    report.passed.extend(verify_status_events(package)?);
    report.passed.extend(verify_batch_summary_totals(package)?);
    report.passed.extend(verify_lineage_conservation(package)?);
    report.passed.extend(verify_rhist_references(package)?);
    report.passed.extend(verify_rctx_references(package)?);
    report.passed.extend(verify_proof_privacy(package)?);
    report
        .passed
        .extend(verify_cvr_summary_reconciliation(package)?);
    report
        .passed
        .extend(verify_batch_comparison_audits(package)?);
    report.passed.extend(verify_audit_algorithm_runs(package)?);
    report.passed.extend(verify_rla_sampler_replay(package)?);
    report.passed.extend(verify_rla_margin_metadata(package)?);
    report.passed.extend(verify_rla_stopping_rules(package)?);
    report
        .passed
        .extend(verify_rla_jurisdiction_adapters(package)?);
    report.passed.extend(verify_manual_audits(package)?);
    Ok(report)
}

pub fn verify_contest_selection_sum(
    contest: &Contest,
    summary: &Summary,
) -> Result<(), RcountCoreError> {
    ensure_non_negative(summary.undervotes)?;
    ensure_non_negative(summary.overvotes)?;
    ensure_non_negative(summary.blank_contests)?;
    ensure_non_negative(summary.counted_ballots)?;

    let valid_selection_ids: BTreeSet<&str> = contest
        .selections
        .iter()
        .map(|selection| selection.selection_id.as_str())
        .collect();
    let mut seen = BTreeSet::new();
    let mut selection_votes = 0i64;
    for total in summary.totals.iter() {
        ensure_non_negative(total.votes)?;
        if !seen.insert(total.selection_id.as_str()) {
            return Err(RcountCoreError::DuplicateSummarySelection {
                contest_id: summary.contest_id.clone(),
                reporting_unit_id: summary.reporting_unit_id.clone(),
                selection_id: total.selection_id.clone(),
            });
        }
        if !valid_selection_ids.contains(total.selection_id.as_str()) {
            return Err(RcountCoreError::UnknownSelection {
                contest_id: summary.contest_id.clone(),
                reporting_unit_id: summary.reporting_unit_id.clone(),
                selection_id: total.selection_id.clone(),
            });
        }
        selection_votes += total.votes;
    }

    let computed =
        selection_votes + summary.undervotes + summary.overvotes + summary.blank_contests;
    if computed != summary.counted_ballots {
        return Err(RcountCoreError::ContestSelectionSumMismatch {
            contest_id: summary.contest_id.clone(),
            reporting_unit_id: summary.reporting_unit_id.clone(),
            declared_ballots: summary.counted_ballots,
            computed_ballots: computed,
        });
    }
    Ok(())
}

pub fn verify_jurisdiction_total(
    contest_id: &str,
    jurisdiction_reporting_unit_id: &str,
    summaries: &[Summary],
) -> Result<Vec<EquationPass>, RcountCoreError> {
    let totals: Vec<&Summary> = summaries
        .iter()
        .filter(|summary| {
            summary.contest_id == contest_id
                && summary.reporting_unit_id == jurisdiction_reporting_unit_id
        })
        .collect();
    if totals.is_empty() {
        return Err(RcountCoreError::MissingJurisdictionTotal {
            contest_id: contest_id.to_string(),
            jurisdiction_reporting_unit_id: jurisdiction_reporting_unit_id.to_string(),
        });
    }

    let mut passes = Vec::new();
    for total in totals {
        verify_jurisdiction_total_for_status(
            contest_id,
            jurisdiction_reporting_unit_id,
            total,
            summaries,
        )?;
        passes.push(EquationPass {
            equation_id: "jurisdiction_contest_total".to_string(),
            contest_id: contest_id.to_string(),
            reporting_unit_id: jurisdiction_reporting_unit_id.to_string(),
        });
    }
    Ok(passes)
}

fn verify_jurisdiction_total_for_status(
    contest_id: &str,
    jurisdiction_reporting_unit_id: &str,
    total: &Summary,
    summaries: &[Summary],
) -> Result<(), RcountCoreError> {
    let mut selection_sums: BTreeMap<&str, i64> = BTreeMap::new();
    let mut undervotes = 0i64;
    let mut overvotes = 0i64;
    let mut blank_contests = 0i64;
    let mut counted_ballots = 0i64;

    for summary in summaries.iter().filter(|summary| {
        summary.contest_id == contest_id
            && summary.reporting_unit_id != jurisdiction_reporting_unit_id
            && summary.status == total.status
    }) {
        for selection in summary.totals.iter() {
            *selection_sums
                .entry(selection.selection_id.as_str())
                .or_default() += selection.votes;
        }
        undervotes += summary.undervotes;
        overvotes += summary.overvotes;
        blank_contests += summary.blank_contests;
        counted_ballots += summary.counted_ballots;
    }

    for total_selection in total.totals.iter() {
        let computed = selection_sums
            .get(total_selection.selection_id.as_str())
            .copied()
            .unwrap_or_default();
        if total_selection.votes != computed {
            return Err(RcountCoreError::JurisdictionSelectionMismatch {
                contest_id: contest_id.to_string(),
                selection_id: total_selection.selection_id.clone(),
                declared_votes: total_selection.votes,
                computed_votes: computed,
            });
        }
    }
    check_residual(contest_id, "undervotes", total.undervotes, undervotes)?;
    check_residual(contest_id, "overvotes", total.overvotes, overvotes)?;
    check_residual(
        contest_id,
        "blank_contests",
        total.blank_contests,
        blank_contests,
    )?;
    check_residual(
        contest_id,
        "counted_ballots",
        total.counted_ballots,
        counted_ballots,
    )?;

    Ok(())
}

pub fn verify_status_events(package: &RcountPackage) -> Result<Vec<EquationPass>, RcountCoreError> {
    let mut seen = BTreeSet::new();
    let mut passes = Vec::new();
    for event in package.status_events.iter() {
        if !seen.insert(event.event_id.as_str()) {
            return Err(RcountCoreError::DuplicateStatusEventId {
                event_id: event.event_id.clone(),
            });
        }
        if event.status_before == event.status_after
            && event.event_type != StatusEventType::InitialUnofficialReport
        {
            return Err(RcountCoreError::NoStatusTransition {
                event_id: event.event_id.clone(),
            });
        }
        if event.authority.trim().is_empty() || event.explanation.trim().is_empty() {
            return Err(RcountCoreError::IncompleteStatusEvent {
                event_id: event.event_id.clone(),
            });
        }
        passes.push(EquationPass {
            equation_id: "status_event_declared".to_string(),
            contest_id: "*".to_string(),
            reporting_unit_id: event.event_id.clone(),
        });
    }
    Ok(passes)
}

pub fn verify_canvass_correction_event(
    package: &RcountPackage,
) -> Result<EquationPass, RcountCoreError> {
    let has_event = package.status_events.iter().any(|event| {
        event.event_type == StatusEventType::Correction
            && event.status_before == CountStatus::Unofficial
            && event.status_after == CountStatus::Canvassed
    });
    if !has_event {
        return Err(RcountCoreError::MissingCanvassCorrectionEvent);
    }
    for status in [CountStatus::Unofficial, CountStatus::Canvassed] {
        if !package
            .summaries
            .iter()
            .any(|summary| summary.status == status)
        {
            return Err(RcountCoreError::MissingStatusSummaries { status });
        }
    }
    Ok(EquationPass {
        equation_id: "canvass_correction_event".to_string(),
        contest_id: "*".to_string(),
        reporting_unit_id: "*".to_string(),
    })
}

pub fn verify_batch_summary_totals(
    package: &RcountPackage,
) -> Result<Vec<EquationPass>, RcountCoreError> {
    let mut batches: BTreeMap<&str, &BatchManifest> = BTreeMap::new();
    let mut passes = Vec::new();
    for batch in package.batches.iter() {
        ensure_non_negative(batch.accepted_ballots)?;
        ensure_non_negative(batch.counted_ballots)?;
        ensure_non_negative(batch.rejected_ballots)?;
        if batches.insert(batch.batch_id.as_str(), batch).is_some() {
            return Err(RcountCoreError::DuplicateBatchId {
                batch_id: batch.batch_id.clone(),
            });
        }
        let computed = batch.counted_ballots + batch.rejected_ballots;
        if batch.accepted_ballots != computed {
            return Err(RcountCoreError::AcceptedBallotsMismatch {
                batch_id: batch.batch_id.clone(),
                declared_ballots: batch.accepted_ballots,
                computed_ballots: computed,
            });
        }
        passes.push(EquationPass {
            equation_id: "accepted_ballots".to_string(),
            contest_id: "*".to_string(),
            reporting_unit_id: batch.batch_id.clone(),
        });
    }

    for summary in package
        .summaries
        .iter()
        .filter(|summary| summary.batch_id.is_some())
    {
        let batch_id = summary
            .batch_id
            .as_ref()
            .expect("filtered to batch summaries");
        let batch =
            batches
                .get(batch_id.as_str())
                .ok_or_else(|| RcountCoreError::MissingBatch {
                    contest_id: summary.contest_id.clone(),
                    reporting_unit_id: summary.reporting_unit_id.clone(),
                    batch_id: batch_id.clone(),
                })?;
        if batch.counted_ballots != summary.counted_ballots {
            return Err(RcountCoreError::BatchSummaryTotalMismatch {
                batch_id: batch_id.clone(),
                declared_ballots: batch.counted_ballots,
                summary_ballots: summary.counted_ballots,
            });
        }
        passes.push(EquationPass {
            equation_id: "batch_summary_total".to_string(),
            contest_id: summary.contest_id.clone(),
            reporting_unit_id: batch_id.clone(),
        });
    }
    Ok(passes)
}

pub fn verify_lineage_conservation(
    package: &RcountPackage,
) -> Result<Vec<EquationPass>, RcountCoreError> {
    let units: BTreeSet<&str> = package
        .reporting_units
        .iter()
        .map(|unit| unit.reporting_unit_id.as_str())
        .collect();
    let mut seen = BTreeSet::new();
    let mut passes = Vec::new();

    for event in package.lineage.iter() {
        if !seen.insert(event.lineage_id.as_str()) {
            return Err(RcountCoreError::DuplicateLineageId {
                lineage_id: event.lineage_id.clone(),
            });
        }
        for prior in event.prior_reporting_unit_ids.iter() {
            if !units.contains(prior.as_str()) {
                return Err(RcountCoreError::MissingPriorLineageUnit {
                    lineage_id: event.lineage_id.clone(),
                    reporting_unit_id: prior.clone(),
                });
            }
        }
        for current in event.current_reporting_unit_ids.iter() {
            if !units.contains(current.as_str()) {
                return Err(RcountCoreError::MissingCurrentLineageUnit {
                    lineage_id: event.lineage_id.clone(),
                    reporting_unit_id: current.clone(),
                });
            }
        }
        match event.kind {
            LineageKind::Unchanged => {
                if event.prior_reporting_unit_ids.len() != 1
                    || event.current_reporting_unit_ids.len() != 1
                {
                    return Err(RcountCoreError::InvalidSplitLineage {
                        lineage_id: event.lineage_id.clone(),
                    });
                }
            }
            LineageKind::Split => {
                if event.prior_reporting_unit_ids.len() != 1
                    || event.current_reporting_unit_ids.len() < 2
                {
                    return Err(RcountCoreError::InvalidSplitLineage {
                        lineage_id: event.lineage_id.clone(),
                    });
                }
            }
            LineageKind::Merge => {
                if event.prior_reporting_unit_ids.len() < 2
                    || event.current_reporting_unit_ids.len() != 1
                {
                    return Err(RcountCoreError::InvalidMergeLineage {
                        lineage_id: event.lineage_id.clone(),
                    });
                }
            }
        }
        passes.push(EquationPass {
            equation_id: "lineage_conservation".to_string(),
            contest_id: "*".to_string(),
            reporting_unit_id: event.lineage_id.clone(),
        });
    }
    Ok(passes)
}

pub fn verify_rhist_references(
    package: &RcountPackage,
) -> Result<Vec<EquationPass>, RcountCoreError> {
    let mut seen = BTreeSet::new();
    let mut passes = Vec::new();
    for reference in &package.rhist_refs {
        if !seen.insert(reference.reference_id.as_str()) {
            return Err(RcountCoreError::DuplicateRhistReference {
                reference_id: reference.reference_id.clone(),
            });
        }
        if !is_sha256_hash(&reference.package_hash) {
            return Err(RcountCoreError::InvalidRhistPackageHash {
                reference_id: reference.reference_id.clone(),
                package_hash: reference.package_hash.clone(),
            });
        }
        if reference.cycle_ids.is_empty() {
            return Err(RcountCoreError::EmptyRhistCycleRefs {
                reference_id: reference.reference_id.clone(),
            });
        }
        if !matches!(
            reference.role.as_str(),
            "unit-lineage" | "aggregation-crosswalk" | "context-lineage"
        ) {
            return Err(RcountCoreError::UnsupportedRhistReferenceRole {
                reference_id: reference.reference_id.clone(),
                role: reference.role.clone(),
            });
        }
        passes.push(EquationPass {
            equation_id: "rhist_reference_declared".to_string(),
            contest_id: "*".to_string(),
            reporting_unit_id: reference.reference_id.clone(),
        });
    }
    Ok(passes)
}

pub fn verify_rctx_references(
    package: &RcountPackage,
) -> Result<Vec<EquationPass>, RcountCoreError> {
    let mut seen = BTreeSet::new();
    let mut passes = Vec::new();
    for reference in &package.rctx_refs {
        if !seen.insert(reference.reference_id.as_str()) {
            return Err(RcountCoreError::DuplicateRctxReference {
                reference_id: reference.reference_id.clone(),
            });
        }
        if !is_sha256_hash(&reference.context_hash) {
            return Err(RcountCoreError::InvalidRctxContextHash {
                reference_id: reference.reference_id.clone(),
                context_hash: reference.context_hash.clone(),
            });
        }
        if let Some(crosswalk_hash) = &reference.crosswalk_hash {
            if !is_sha256_hash(crosswalk_hash) {
                return Err(RcountCoreError::InvalidRctxCrosswalkHash {
                    reference_id: reference.reference_id.clone(),
                    crosswalk_hash: crosswalk_hash.clone(),
                });
            }
        }
        if !matches!(
            reference.role.as_str(),
            "unit-context" | "aggregation-crosswalk" | "plan-context"
        ) {
            return Err(RcountCoreError::UnsupportedRctxReferenceRole {
                reference_id: reference.reference_id.clone(),
                role: reference.role.clone(),
            });
        }
        passes.push(EquationPass {
            equation_id: "rctx_reference_declared".to_string(),
            contest_id: "*".to_string(),
            reporting_unit_id: reference.reference_id.clone(),
        });
    }
    Ok(passes)
}

pub fn verify_proof_privacy(package: &RcountPackage) -> Result<Vec<EquationPass>, RcountCoreError> {
    let mut seen = BTreeSet::new();
    let mut passes = Vec::new();
    for proof in package.inclusion_proofs.iter() {
        if !seen.insert(proof.proof_id.as_str()) {
            return Err(RcountCoreError::DuplicateProofId {
                proof_id: proof.proof_id.clone(),
            });
        }
        if !is_sha256_hash(&proof.token_hash) {
            return Err(RcountCoreError::InvalidProofTokenHash {
                proof_id: proof.proof_id.clone(),
                token_hash: proof.token_hash.clone(),
            });
        }
        if !proof.candidate_selections.is_empty() {
            return Err(RcountCoreError::ChoiceBearingProof {
                proof_id: proof.proof_id.clone(),
            });
        }
        if proof.voter_id.is_some() && proof.ballot_style.is_some() && proof.issued_at.is_some() {
            return Err(RcountCoreError::LinkableVoterProof {
                proof_id: proof.proof_id.clone(),
            });
        }
        passes.push(EquationPass {
            equation_id: "proof_privacy_gate".to_string(),
            contest_id: "*".to_string(),
            reporting_unit_id: proof.proof_id.clone(),
        });
    }
    Ok(passes)
}

pub fn verify_cvr_summary_reconciliation(
    package: &RcountPackage,
) -> Result<Vec<EquationPass>, RcountCoreError> {
    if package.cvr.is_empty() {
        return Ok(Vec::new());
    }

    let contests: BTreeMap<&str, &Contest> = package
        .contests
        .iter()
        .map(|contest| (contest.contest_id.as_str(), contest))
        .collect();
    let mut seen = BTreeSet::new();
    let mut aggregates: BTreeMap<CvrAggregateKey, CvrAggregate> = BTreeMap::new();

    for row in &package.cvr {
        if !seen.insert((row.cvr_id.as_str(), row.contest_id.as_str())) {
            return Err(RcountCoreError::DuplicateCvrContest {
                cvr_id: row.cvr_id.clone(),
                contest_id: row.contest_id.clone(),
            });
        }
        let contest = contests.get(row.contest_id.as_str()).ok_or_else(|| {
            RcountCoreError::UnknownCvrSelection {
                cvr_id: row.cvr_id.clone(),
                contest_id: row.contest_id.clone(),
                selection_id: "<contest-missing>".to_string(),
            }
        })?;
        let valid_selection_ids: BTreeSet<&str> = contest
            .selections
            .iter()
            .map(|selection| selection.selection_id.as_str())
            .collect();
        let residual_count = row.undervote as u8 + row.overvote as u8 + row.blank_contest as u8;
        if residual_count > 1
            || (residual_count == 1 && !row.selection_ids.is_empty())
            || row.selection_ids.len() > contest.vote_for as usize
        {
            return Err(RcountCoreError::InvalidCvrContestCardinality {
                cvr_id: row.cvr_id.clone(),
                contest_id: row.contest_id.clone(),
            });
        }
        for selection_id in &row.selection_ids {
            if !valid_selection_ids.contains(selection_id.as_str()) {
                return Err(RcountCoreError::UnknownCvrSelection {
                    cvr_id: row.cvr_id.clone(),
                    contest_id: row.contest_id.clone(),
                    selection_id: selection_id.clone(),
                });
            }
        }

        let aggregate = aggregates.entry(CvrAggregateKey::from(row)).or_default();
        aggregate.counted_ballots += 1;
        for selection_id in &row.selection_ids {
            *aggregate
                .selection_votes
                .entry(selection_id.clone())
                .or_default() += 1;
        }
        aggregate.undervotes += row.undervote as i64;
        aggregate.overvotes += row.overvote as i64;
        aggregate.blank_contests += row.blank_contest as i64;
    }

    let mut passes = Vec::new();
    for (key, aggregate) in aggregates {
        let summary = package
            .summaries
            .iter()
            .find(|summary| {
                summary.contest_id == key.contest_id
                    && summary.reporting_unit_id == key.reporting_unit_id
                    && summary.batch_id == key.batch_id
                    && summary.status == key.status
            })
            .ok_or_else(|| RcountCoreError::MissingCvrSummary {
                contest_id: key.contest_id.clone(),
                reporting_unit_id: key.reporting_unit_id.clone(),
            })?;

        for total in &summary.totals {
            let cvr = aggregate
                .selection_votes
                .get(&total.selection_id)
                .copied()
                .unwrap_or_default();
            check_cvr_field(
                &key.contest_id,
                &key.reporting_unit_id,
                &format!("selection:{}", total.selection_id),
                total.votes,
                cvr,
            )?;
        }
        check_cvr_field(
            &key.contest_id,
            &key.reporting_unit_id,
            "undervotes",
            summary.undervotes,
            aggregate.undervotes,
        )?;
        check_cvr_field(
            &key.contest_id,
            &key.reporting_unit_id,
            "overvotes",
            summary.overvotes,
            aggregate.overvotes,
        )?;
        check_cvr_field(
            &key.contest_id,
            &key.reporting_unit_id,
            "blank_contests",
            summary.blank_contests,
            aggregate.blank_contests,
        )?;
        check_cvr_field(
            &key.contest_id,
            &key.reporting_unit_id,
            "counted_ballots",
            summary.counted_ballots,
            aggregate.counted_ballots,
        )?;
        passes.push(EquationPass {
            equation_id: "cvr_summary_total".to_string(),
            contest_id: key.contest_id,
            reporting_unit_id: key.reporting_unit_id,
        });
    }
    Ok(passes)
}

pub fn verify_rla_sampler_replay(
    package: &RcountPackage,
) -> Result<Vec<EquationPass>, RcountCoreError> {
    let mut seen = BTreeSet::new();
    let mut passes = Vec::new();
    for audit in &package.rla_audits {
        if !seen.insert(audit.audit_id.as_str()) {
            return Err(RcountCoreError::DuplicateRlaAuditId {
                audit_id: audit.audit_id.clone(),
            });
        }
        if audit.risk_limit_ppm == 0 || audit.risk_limit_ppm >= 1_000_000 {
            return Err(RcountCoreError::InvalidRlaRiskLimit {
                audit_id: audit.audit_id.clone(),
                risk_limit_ppm: audit.risk_limit_ppm,
            });
        }
        if audit.sample_size == 0 || audit.sample_draws.len() != audit.sample_size as usize {
            return Err(RcountCoreError::InvalidRlaSampleSize {
                audit_id: audit.audit_id.clone(),
                sample_size: audit.sample_size,
            });
        }
        if audit.sampling_algorithm_id != RLA_SAMPLING_ALGORITHM_ID {
            return Err(RcountCoreError::UnsupportedRlaSamplingAlgorithm {
                audit_id: audit.audit_id.clone(),
                sampling_algorithm_id: audit.sampling_algorithm_id.clone(),
            });
        }
        let computed_manifest_hash =
            rla_contest_manifest_hash_for_audit(package, &audit.contest_id, &audit.audit_id)?;
        if audit.manifest_hash != computed_manifest_hash {
            return Err(RcountCoreError::RlaManifestHashMismatch {
                audit_id: audit.audit_id.clone(),
                declared: audit.manifest_hash.clone(),
                computed: computed_manifest_hash,
            });
        }
        let expected = replay_rla_sample(package, audit)?;
        for (declared, computed) in audit.sample_draws.iter().zip(expected.iter()) {
            if declared.draw_index != computed.draw_index || declared.cvr_id != computed.cvr_id {
                return Err(RcountCoreError::RlaSampleMismatch {
                    audit_id: audit.audit_id.clone(),
                    draw_index: computed.draw_index,
                    declared_cvr_id: declared.cvr_id.clone(),
                    computed_cvr_id: computed.cvr_id.clone(),
                });
            }
        }
        passes.push(EquationPass {
            equation_id: "rla_sampler_replay".to_string(),
            contest_id: audit.contest_id.clone(),
            reporting_unit_id: audit.audit_id.clone(),
        });
    }
    Ok(passes)
}

pub fn verify_audit_algorithm_runs(
    package: &RcountPackage,
) -> Result<Vec<EquationPass>, RcountCoreError> {
    let mut seen_runs = BTreeSet::new();
    for run in &package.audit_algorithm_runs {
        if !seen_runs.insert(run.run_id.as_str()) {
            return Err(RcountCoreError::DuplicateAuditAlgorithmRunId {
                run_id: run.run_id.clone(),
            });
        }
    }

    let mut passes = Vec::new();
    for run in &package.audit_algorithm_runs {
        if !is_supported_audit_algorithm_method(&run.method_id) {
            return Err(RcountCoreError::UnsupportedAuditAlgorithmMethod {
                run_id: run.run_id.clone(),
                method_id: run.method_id.clone(),
            });
        }
        if let Some(risk_limit_ppm) = run.risk_limit_ppm {
            if risk_limit_ppm == 0 || risk_limit_ppm >= 1_000_000 {
                return Err(RcountCoreError::InvalidAuditAlgorithmRiskLimit {
                    run_id: run.run_id.clone(),
                    risk_limit_ppm,
                });
            }
        }
        verify_audit_macro_design(run)?;
        verify_stratified_hybrid_design(run, &seen_runs)?;
        verify_ranked_choice_audit_design(run)?;
        verify_bayesian_audit_design(run)?;
        verify_soba_observable_ballot_design(package, run)?;

        let mut assertions = BTreeMap::new();
        for assertion in &run.assertions {
            if assertions
                .insert(assertion.assertion_id.as_str(), assertion)
                .is_some()
            {
                return Err(RcountCoreError::DuplicateAuditAssertion {
                    run_id: run.run_id.clone(),
                    assertion_id: assertion.assertion_id.clone(),
                });
            }
            if assertion.assorter_id.trim().is_empty()
                || !is_positive_rational(assertion.assorter_upper_bound)
            {
                return Err(RcountCoreError::InvalidAuditAssorterBound {
                    run_id: run.run_id.clone(),
                    assertion_id: assertion.assertion_id.clone(),
                });
            }
        }

        let mut seen_steps = BTreeSet::new();
        for step in &run.sample_steps {
            let Some(assertion) = assertions.get(step.assertion_id.as_str()) else {
                return Err(RcountCoreError::MissingAuditAssertion {
                    run_id: run.run_id.clone(),
                    step_index: step.step_index,
                    assertion_id: step.assertion_id.clone(),
                });
            };
            if !seen_steps.insert((step.assertion_id.as_str(), step.step_index)) {
                return Err(RcountCoreError::DuplicateAuditSampleStep {
                    run_id: run.run_id.clone(),
                    assertion_id: step.assertion_id.clone(),
                    step_index: step.step_index,
                });
            }
            if !is_non_negative_rational(step.assorter_value)
                || rational_gt(step.assorter_value, assertion.assorter_upper_bound)
                || step.bet.is_some_and(|bet| !has_positive_denominator(bet))
                || step
                    .statistic
                    .is_some_and(|statistic| !is_non_negative_rational(statistic))
            {
                return Err(RcountCoreError::InvalidAuditAssorterValue {
                    run_id: run.run_id.clone(),
                    step_index: step.step_index,
                });
            }
            if step.p_value_ppm.is_some_and(|p_value| p_value > 1_000_000) {
                return Err(RcountCoreError::InvalidAuditPValue {
                    run_id: run.run_id.clone(),
                    step_index: step.step_index,
                    p_value_ppm: step.p_value_ppm.unwrap(),
                });
            }
        }

        passes.push(EquationPass {
            equation_id: "audit_algorithm_transcript".to_string(),
            contest_id: run.contest_id.clone(),
            reporting_unit_id: run.run_id.clone(),
        });
        if run.method_id == BATCH_COMPARISON_METHOD_ID
            && package
                .batch_comparison_audits
                .iter()
                .any(|audit| audit.contest_id == run.contest_id)
        {
            verify_batch_comparison_algorithm_linkage(package, run)?;
            passes.push(EquationPass {
                equation_id: "batch_comparison_algorithm_linkage".to_string(),
                contest_id: run.contest_id.clone(),
                reporting_unit_id: run.run_id.clone(),
            });
        }
    }
    Ok(passes)
}

fn verify_audit_macro_design(run: &AuditAlgorithmRun) -> Result<(), RcountCoreError> {
    match (
        run.macro_ballot_count,
        run.macro_reported_margin,
        run.macro_gamma,
    ) {
        (None, None, None) => Ok(()),
        (Some(ballot_count), Some(reported_margin), Some(gamma))
            if ballot_count > 0
                && reported_margin > 0
                && gamma.denominator > 0
                && rational_gt(
                    gamma,
                    RationalValue {
                        numerator: 1,
                        denominator: 1,
                    },
                ) =>
        {
            Ok(())
        }
        _ => Err(RcountCoreError::InvalidAuditMacroDesign {
            run_id: run.run_id.clone(),
        }),
    }
}

fn verify_stratified_hybrid_design(
    run: &AuditAlgorithmRun,
    run_ids: &BTreeSet<&str>,
) -> Result<(), RcountCoreError> {
    if run.method_id != STRATIFIED_HYBRID_RLA_METHOD_ID {
        if run.combining_rule_id.is_some()
            || run.nuisance_parameter.is_some()
            || !run.strata.is_empty()
        {
            return Err(RcountCoreError::InvalidStratifiedHybridDesign {
                run_id: run.run_id.clone(),
            });
        }
        return Ok(());
    }

    if run
        .combining_rule_id
        .as_deref()
        .is_none_or(|rule| rule.trim().is_empty())
        || run
            .nuisance_parameter
            .is_none_or(|parameter| !has_positive_denominator(parameter))
        || run.strata.len() < 2
        || !run.assertions.is_empty()
        || !run.sample_steps.is_empty()
    {
        return Err(RcountCoreError::InvalidStratifiedHybridDesign {
            run_id: run.run_id.clone(),
        });
    }

    let mut seen_strata = BTreeSet::new();
    let mut allocation_sum = 0_u32;
    for stratum in &run.strata {
        let Some(allocation_ppm) = stratum.allocation_ppm else {
            return Err(RcountCoreError::InvalidStratifiedHybridDesign {
                run_id: run.run_id.clone(),
            });
        };
        if stratum.stratum_id.trim().is_empty()
            || !seen_strata.insert(stratum.stratum_id.as_str())
            || stratum.component_run_id == run.run_id
            || !is_supported_audit_algorithm_method(&stratum.method_id)
            || stratum.method_id == STRATIFIED_HYBRID_RLA_METHOD_ID
            || stratum.ballot_count.is_some_and(|ballots| ballots == 0)
            || allocation_ppm == 0
            || allocation_ppm > 1_000_000
        {
            return Err(RcountCoreError::InvalidStratifiedHybridDesign {
                run_id: run.run_id.clone(),
            });
        }
        allocation_sum = allocation_sum.saturating_add(allocation_ppm);
        if !run_ids.contains(stratum.component_run_id.as_str()) {
            return Err(RcountCoreError::MissingStratifiedHybridComponent {
                run_id: run.run_id.clone(),
                component_run_id: stratum.component_run_id.clone(),
            });
        }
    }
    if allocation_sum != 1_000_000 {
        return Err(RcountCoreError::InvalidStratifiedHybridDesign {
            run_id: run.run_id.clone(),
        });
    }
    Ok(())
}

fn verify_ranked_choice_audit_design(run: &AuditAlgorithmRun) -> Result<(), RcountCoreError> {
    let is_ranked_method = matches!(
        run.method_id.as_str(),
        RAIRE_IRV_METHOD_ID | AWAIRE_IRV_METHOD_ID
    );
    if !is_ranked_method {
        if !run.rcv_elimination_order.is_empty()
            || run
                .sample_steps
                .iter()
                .any(|step| !step.ranked_choices.is_empty())
        {
            return Err(RcountCoreError::InvalidRankedChoiceAuditDesign {
                run_id: run.run_id.clone(),
            });
        }
        return Ok(());
    }

    if run.rcv_elimination_order.len() < 2
        || run.assertions.is_empty()
        || run
            .assertions
            .iter()
            .any(|assertion| assertion.kind != AuditAssertionKind::RankedChoiceAssertion)
    {
        return Err(RcountCoreError::InvalidRankedChoiceAuditDesign {
            run_id: run.run_id.clone(),
        });
    }

    let candidates = run
        .rcv_elimination_order
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    if candidates.len() != run.rcv_elimination_order.len() {
        return Err(RcountCoreError::InvalidRankedChoiceAuditDesign {
            run_id: run.run_id.clone(),
        });
    }

    for step in &run.sample_steps {
        if step.ranked_choices.is_empty() {
            return Err(RcountCoreError::InvalidRankedChoiceSample {
                run_id: run.run_id.clone(),
                step_index: step.step_index,
            });
        }
        let mut seen_choices = BTreeSet::new();
        for choice in &step.ranked_choices {
            if !candidates.contains(choice.as_str()) || !seen_choices.insert(choice.as_str()) {
                return Err(RcountCoreError::InvalidRankedChoiceSample {
                    run_id: run.run_id.clone(),
                    step_index: step.step_index,
                });
            }
        }
    }

    Ok(())
}

fn verify_bayesian_audit_design(run: &AuditAlgorithmRun) -> Result<(), RcountCoreError> {
    if run.method_id != BAYESIAN_TABULATION_AUDIT_METHOD_ID {
        if run.bayesian_prior_id.is_some()
            || run.bayesian_likelihood_id.is_some()
            || run.posterior_winner_probability_ppm.is_some()
            || run.posterior_risk_ppm.is_some()
            || run.simulation_seed.is_some()
            || run.posterior_draws.is_some()
            || run.calibrated_risk_limit_ppm.is_some()
        {
            return Err(RcountCoreError::InvalidBayesianAuditDesign {
                run_id: run.run_id.clone(),
            });
        }
        return Ok(());
    }

    if run
        .bayesian_prior_id
        .as_deref()
        .is_none_or(|prior| prior.trim().is_empty())
        || run
            .bayesian_likelihood_id
            .as_deref()
            .is_none_or(|likelihood| likelihood.trim().is_empty())
        || run
            .posterior_winner_probability_ppm
            .is_none_or(|probability| probability > 1_000_000)
        || run.posterior_risk_ppm.is_none_or(|risk| risk > 1_000_000)
        || run.posterior_draws.is_some_and(|draws| draws == 0)
        || run
            .calibrated_risk_limit_ppm
            .is_some_and(|limit| limit == 0 || limit >= 1_000_000)
        || run.assertions.is_empty()
        || run
            .assertions
            .iter()
            .any(|assertion| assertion.kind != AuditAssertionKind::BayesianOutcome)
    {
        return Err(RcountCoreError::InvalidBayesianAuditDesign {
            run_id: run.run_id.clone(),
        });
    }

    Ok(())
}

fn verify_soba_observable_ballot_design(
    package: &RcountPackage,
    run: &AuditAlgorithmRun,
) -> Result<(), RcountCoreError> {
    if run.method_id != SOBA_OBSERVABLE_BALLOT_AUDIT_METHOD_ID {
        if run
            .assertions
            .iter()
            .any(|assertion| assertion.kind == AuditAssertionKind::ObservableBallotLinkage)
        {
            return Err(RcountCoreError::InvalidObservableBallotAuditDesign {
                run_id: run.run_id.clone(),
            });
        }
        return Ok(());
    }

    if run.assertions.is_empty()
        || run.sample_steps.is_empty()
        || run
            .assertions
            .iter()
            .any(|assertion| assertion.kind != AuditAssertionKind::ObservableBallotLinkage)
    {
        return Err(RcountCoreError::InvalidObservableBallotAuditDesign {
            run_id: run.run_id.clone(),
        });
    }

    let proofs = package
        .inclusion_proofs
        .iter()
        .map(|proof| (proof.proof_id.as_str(), proof))
        .collect::<BTreeMap<_, _>>();
    for step in &run.sample_steps {
        let proof = proofs.get(step.sample_unit_id.as_str()).ok_or_else(|| {
            RcountCoreError::MissingObservableBallotOpening {
                run_id: run.run_id.clone(),
                step_index: step.step_index,
                proof_id: step.sample_unit_id.clone(),
            }
        })?;
        if !proof.candidate_selections.is_empty() || proof.voter_id.is_some() {
            return Err(RcountCoreError::InvalidObservableBallotAuditDesign {
                run_id: run.run_id.clone(),
            });
        }
    }

    Ok(())
}

fn verify_batch_comparison_algorithm_linkage(
    package: &RcountPackage,
    run: &AuditAlgorithmRun,
) -> Result<(), RcountCoreError> {
    let audits_by_batch = package
        .batch_comparison_audits
        .iter()
        .filter(|audit| audit.contest_id == run.contest_id)
        .map(|audit| (audit.batch_id.as_str(), audit))
        .collect::<BTreeMap<_, _>>();

    for step in &run.sample_steps {
        let audit = audits_by_batch
            .get(step.sample_unit_id.as_str())
            .ok_or_else(
                || RcountCoreError::MissingBatchComparisonAlgorithmEvidence {
                    run_id: run.run_id.clone(),
                    step_index: step.step_index,
                    batch_id: step.sample_unit_id.clone(),
                },
            )?;
        let computed = RationalValue {
            numerator: audit.declared_overstatement,
            denominator: audit.declared_reported_margin,
        };
        if !rational_eq(step.assorter_value, computed) {
            return Err(RcountCoreError::BatchComparisonAlgorithmTaintMismatch {
                run_id: run.run_id.clone(),
                step_index: step.step_index,
                declared: step.assorter_value,
                computed,
            });
        }
    }

    Ok(())
}

pub fn derive_batch_comparison_algorithm_run(
    package: &RcountPackage,
    run_id: &str,
    contest_id: &str,
    risk_limit_ppm: u32,
    sampled_batch_ids: &[String],
    decision: AuditAlgorithmDecision,
) -> Result<AuditAlgorithmRun, RcountCoreError> {
    if sampled_batch_ids.is_empty() {
        return Err(RcountCoreError::EmptyBatchComparisonAlgorithmSample {
            run_id: run_id.to_string(),
        });
    }
    if risk_limit_ppm == 0 || risk_limit_ppm >= 1_000_000 {
        return Err(RcountCoreError::InvalidAuditAlgorithmRiskLimit {
            run_id: run_id.to_string(),
            risk_limit_ppm,
        });
    }

    verify_batch_comparison_audits(package)?;

    let audits_by_batch = package
        .batch_comparison_audits
        .iter()
        .filter(|audit| audit.contest_id == contest_id)
        .map(|audit| (audit.batch_id.as_str(), audit))
        .collect::<BTreeMap<_, _>>();

    let first_audit = audits_by_batch
        .get(sampled_batch_ids[0].as_str())
        .ok_or_else(
            || RcountCoreError::MissingBatchComparisonAlgorithmEvidence {
                run_id: run_id.to_string(),
                step_index: 0,
                batch_id: sampled_batch_ids[0].clone(),
            },
        )?;
    let winner_selection_id = first_audit.winner_selection_id.clone();
    let loser_selection_id = first_audit.loser_selection_id.clone();
    let assertion_id = format!("assertion:{winner_selection_id}-over-{loser_selection_id}");

    let mut run_source_refs = BTreeSet::new();
    let mut sample_steps = Vec::with_capacity(sampled_batch_ids.len());
    for (step_index, batch_id) in sampled_batch_ids.iter().enumerate() {
        let audit = audits_by_batch.get(batch_id.as_str()).ok_or_else(|| {
            RcountCoreError::MissingBatchComparisonAlgorithmEvidence {
                run_id: run_id.to_string(),
                step_index: step_index as u32,
                batch_id: batch_id.clone(),
            }
        })?;
        if audit.winner_selection_id != winner_selection_id
            || audit.loser_selection_id != loser_selection_id
        {
            return Err(RcountCoreError::BatchComparisonAlgorithmAssertionMismatch {
                run_id: run_id.to_string(),
                audit_id: audit.audit_id.clone(),
            });
        }
        if audit.declared_reported_margin <= 0 {
            return Err(RcountCoreError::InvalidBatchComparisonAlgorithmMargin {
                run_id: run_id.to_string(),
                audit_id: audit.audit_id.clone(),
                reported_margin: audit.declared_reported_margin,
            });
        }

        let mut step_source_refs = vec![audit.audit_id.clone()];
        step_source_refs.extend(audit.source_refs.clone());
        run_source_refs.extend(step_source_refs.iter().cloned());
        sample_steps.push(AuditSampleStep {
            step_index: step_index as u32,
            round_index: None,
            assertion_id: assertion_id.clone(),
            sample_unit_id: batch_id.clone(),
            assorter_value: RationalValue {
                numerator: audit.declared_overstatement,
                denominator: audit.declared_reported_margin,
            },
            bet: None,
            statistic: None,
            p_value_ppm: None,
            ranked_choices: Vec::new(),
            source_refs: step_source_refs,
        });
    }

    Ok(AuditAlgorithmRun {
        run_id: run_id.to_string(),
        contest_id: contest_id.to_string(),
        method_id: BATCH_COMPARISON_METHOD_ID.to_string(),
        sampling_mode: AuditSamplingMode::Batch,
        rcv_elimination_order: Vec::new(),
        risk_limit_ppm: Some(risk_limit_ppm),
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
            assertion_id,
            kind: AuditAssertionKind::ComparisonOverstatement,
            assorter_id: "batch-plurality-overstatement-taint-v1".to_string(),
            assorter_upper_bound: RationalValue {
                numerator: 1,
                denominator: 1,
            },
            winner_selection_id: Some(winner_selection_id),
            loser_selection_id: Some(loser_selection_id),
        }],
        sample_steps,
        decision,
        source_refs: run_source_refs.into_iter().collect(),
    })
}

pub fn rla_contest_manifest_hash(
    package: &RcountPackage,
    contest_id: &str,
) -> Result<String, RcountCoreError> {
    rla_contest_manifest_hash_for_audit(package, contest_id, "<manifest-hash>")
}

fn rla_contest_manifest_hash_for_audit(
    package: &RcountPackage,
    contest_id: &str,
    audit_id: &str,
) -> Result<String, RcountCoreError> {
    let population = rla_population(package, contest_id);
    if population.is_empty() {
        return Err(RcountCoreError::MissingRlaPopulation {
            audit_id: audit_id.to_string(),
            contest_id: contest_id.to_string(),
        });
    }
    let value = serde_json::json!({
        "contest_id": contest_id,
        "cvr_ids": population,
    });
    canonical_hash(RLA_MANIFEST_HASH_PREFIX, &value)
}

pub fn replay_rla_sample(
    package: &RcountPackage,
    audit: &RiskLimitAudit,
) -> Result<Vec<RlaSampleDraw>, RcountCoreError> {
    let population = rla_population(package, &audit.contest_id);
    if population.is_empty() {
        return Err(RcountCoreError::MissingRlaPopulation {
            audit_id: audit.audit_id.clone(),
            contest_id: audit.contest_id.clone(),
        });
    }

    let mut draws = Vec::with_capacity(audit.sample_size as usize);
    for draw_index in 0..audit.sample_size {
        let mut h = Sha256::new();
        h.update(RLA_SAMPLE_PREFIX);
        h.update(audit.manifest_hash.as_bytes());
        h.update(b"\0");
        h.update(audit.public_seed.as_bytes());
        h.update(b"\0");
        h.update(audit.contest_id.as_bytes());
        h.update(b"\0");
        h.update(audit.risk_limit_ppm.to_le_bytes());
        h.update(draw_index.to_le_bytes());
        h.update(audit.sampling_algorithm_id.as_bytes());
        let digest = h.finalize();
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&digest[..8]);
        let selected = u64::from_le_bytes(bytes) as usize % population.len();
        draws.push(RlaSampleDraw {
            draw_index,
            cvr_id: population[selected].clone(),
        });
    }
    Ok(draws)
}

pub fn verify_rla_margin_metadata(
    package: &RcountPackage,
) -> Result<Vec<EquationPass>, RcountCoreError> {
    let mut passes = Vec::new();
    for audit in &package.rla_audits {
        let Some(margin) = &audit.margin else {
            continue;
        };
        let summary = package
            .summaries
            .iter()
            .find(|summary| {
                summary.contest_id == audit.contest_id
                    && summary.batch_id.is_none()
                    && summary.status == CountStatus::Canvassed
                    && package.reporting_units.iter().any(|unit| {
                        unit.reporting_unit_id == summary.reporting_unit_id
                            && unit.kind == ReportingUnitKind::JurisdictionTotal
                    })
            })
            .ok_or_else(|| RcountCoreError::MissingJurisdictionTotal {
                contest_id: audit.contest_id.clone(),
                jurisdiction_reporting_unit_id: "<jurisdiction-total>".to_string(),
            })?;
        let totals: BTreeMap<&str, i64> = summary
            .totals
            .iter()
            .map(|total| (total.selection_id.as_str(), total.votes))
            .collect();
        let winner_votes = totals
            .get(margin.winner_selection_id.as_str())
            .copied()
            .ok_or_else(|| RcountCoreError::MissingRlaMarginSelection {
                audit_id: audit.audit_id.clone(),
                selection_id: margin.winner_selection_id.clone(),
            })?;
        let loser_votes = totals
            .get(margin.loser_selection_id.as_str())
            .copied()
            .ok_or_else(|| RcountCoreError::MissingRlaMarginSelection {
                audit_id: audit.audit_id.clone(),
                selection_id: margin.loser_selection_id.clone(),
            })?;
        if margin.reported_winner_votes != winner_votes {
            return Err(RcountCoreError::RlaWinnerVotesMismatch {
                audit_id: audit.audit_id.clone(),
                selection_id: margin.winner_selection_id.clone(),
                declared: margin.reported_winner_votes,
                summary: winner_votes,
            });
        }
        if margin.reported_loser_votes != loser_votes {
            return Err(RcountCoreError::RlaLoserVotesMismatch {
                audit_id: audit.audit_id.clone(),
                selection_id: margin.loser_selection_id.clone(),
                declared: margin.reported_loser_votes,
                summary: loser_votes,
            });
        }
        let computed_margin = winner_votes - loser_votes;
        if computed_margin <= 0 {
            return Err(RcountCoreError::InvalidRlaReportedMargin {
                audit_id: audit.audit_id.clone(),
                margin: computed_margin,
            });
        }
        if margin.reported_margin != computed_margin {
            return Err(RcountCoreError::RlaReportedMarginMismatch {
                audit_id: audit.audit_id.clone(),
                declared: margin.reported_margin,
                summary: computed_margin,
            });
        }
        if margin.diluted_margin_denominator != summary.counted_ballots {
            return Err(RcountCoreError::RlaDilutedMarginDenominatorMismatch {
                audit_id: audit.audit_id.clone(),
                declared: margin.diluted_margin_denominator,
                summary: summary.counted_ballots,
            });
        }
        passes.push(EquationPass {
            equation_id: "rla_margin_metadata".to_string(),
            contest_id: audit.contest_id.clone(),
            reporting_unit_id: audit.audit_id.clone(),
        });
    }
    Ok(passes)
}

pub fn verify_rla_stopping_rules(
    package: &RcountPackage,
) -> Result<Vec<EquationPass>, RcountCoreError> {
    let mut passes = Vec::new();
    for audit in &package.rla_audits {
        if audit.observations.is_empty()
            && audit.stopping_rule_id.is_none()
            && audit.max_discrepancies.is_none()
            && audit.declared_status.is_none()
            && audit.declared_risk_ppm.is_none()
        {
            continue;
        }
        let stopping_rule_id = audit.stopping_rule_id.as_deref().ok_or_else(|| {
            RcountCoreError::MissingRlaStoppingRule {
                audit_id: audit.audit_id.clone(),
            }
        })?;
        if !matches!(
            stopping_rule_id,
            "zero-discrepancy-threshold-v1" | "comparison-margin-threshold-v1"
        ) || audit.max_discrepancies.is_none()
            || audit.declared_status.is_none()
        {
            return Err(RcountCoreError::MissingRlaStoppingRule {
                audit_id: audit.audit_id.clone(),
            });
        }
        if stopping_rule_id == "comparison-margin-threshold-v1"
            && (audit.margin.is_none() || audit.declared_risk_ppm.is_none())
        {
            return Err(RcountCoreError::MissingRlaRiskEstimate {
                audit_id: audit.audit_id.clone(),
            });
        }

        let cvr_by_id: BTreeMap<&str, &CvrContestRecord> = package
            .cvr
            .iter()
            .filter(|row| row.contest_id == audit.contest_id)
            .map(|row| (row.cvr_id.as_str(), row))
            .collect();
        let mut observations = BTreeMap::new();
        for observation in &audit.observations {
            if observations
                .insert(observation.draw_index, observation)
                .is_some()
            {
                return Err(RcountCoreError::DuplicateRlaObservation {
                    audit_id: audit.audit_id.clone(),
                    draw_index: observation.draw_index,
                });
            }
        }

        let mut computed_discrepancies = Vec::new();
        for draw in &audit.sample_draws {
            let observation = observations.get(&draw.draw_index).ok_or_else(|| {
                RcountCoreError::MissingRlaObservation {
                    audit_id: audit.audit_id.clone(),
                    draw_index: draw.draw_index,
                }
            })?;
            if observation.cvr_id != draw.cvr_id {
                let discrepancy = RlaDiscrepancy {
                    draw_index: draw.draw_index,
                    cvr_id: draw.cvr_id.clone(),
                    kind: RlaDiscrepancyKind::WrongCvrObserved,
                };
                computed_discrepancies.push(discrepancy);
                continue;
            }
            let cvr = cvr_by_id.get(draw.cvr_id.as_str()).ok_or_else(|| {
                RcountCoreError::MissingRlaPopulation {
                    audit_id: audit.audit_id.clone(),
                    contest_id: audit.contest_id.clone(),
                }
            })?;
            if let Some(kind) = classify_rla_discrepancy(observation, cvr) {
                computed_discrepancies.push(RlaDiscrepancy {
                    draw_index: draw.draw_index,
                    cvr_id: draw.cvr_id.clone(),
                    kind,
                });
            }
        }

        verify_declared_rla_discrepancies(audit, &computed_discrepancies)?;

        let computed_risk_ppm = if stopping_rule_id == "comparison-margin-threshold-v1" {
            let computed = comparison_margin_risk_ppm(audit);
            let declared = audit.declared_risk_ppm.unwrap();
            if declared != computed {
                return Err(RcountCoreError::RlaRiskEstimateMismatch {
                    audit_id: audit.audit_id.clone(),
                    declared_ppm: declared,
                    computed_ppm: computed,
                });
            }
            Some(computed)
        } else {
            None
        };

        let computed = if computed_discrepancies.len() as u32 <= audit.max_discrepancies.unwrap()
            && computed_risk_ppm.map_or(true, |risk| risk <= audit.risk_limit_ppm)
        {
            RlaStoppingStatus::Pass
        } else {
            RlaStoppingStatus::Escalate
        };
        let declared = audit.declared_status.unwrap();
        if declared != computed {
            return Err(RcountCoreError::RlaStoppingStatusMismatch {
                audit_id: audit.audit_id.clone(),
                declared,
                computed,
            });
        }
        passes.push(EquationPass {
            equation_id: "rla_stopping_rule".to_string(),
            contest_id: audit.contest_id.clone(),
            reporting_unit_id: audit.audit_id.clone(),
        });
    }
    Ok(passes)
}

pub fn verify_rla_jurisdiction_adapters(
    package: &RcountPackage,
) -> Result<Vec<EquationPass>, RcountCoreError> {
    let mut passes = Vec::new();
    for audit in &package.rla_audits {
        let Some(jurisdiction_method_id) = audit.jurisdiction_method_id.as_deref() else {
            continue;
        };
        match jurisdiction_method_id {
            COLORADO_RLA_METHOD_ID => verify_colorado_rla_adapter(audit)?,
            CALIFORNIA_RLA_METHOD_ID => verify_california_rla_adapter(audit)?,
            other => {
                return Err(RcountCoreError::UnsupportedRlaJurisdictionMethod {
                    audit_id: audit.audit_id.clone(),
                    jurisdiction_method_id: other.to_string(),
                });
            }
        }
        passes.push(EquationPass {
            equation_id: "rla_jurisdiction_adapter".to_string(),
            contest_id: audit.contest_id.clone(),
            reporting_unit_id: audit.audit_id.clone(),
        });
    }
    Ok(passes)
}

fn verify_colorado_rla_adapter(audit: &RiskLimitAudit) -> Result<(), RcountCoreError> {
    if audit.public_seed.len() != 20 || !audit.public_seed.bytes().all(|byte| byte.is_ascii_digit())
    {
        return Err(RcountCoreError::InvalidColoradoRlaSeed {
            audit_id: audit.audit_id.clone(),
            public_seed: audit.public_seed.clone(),
        });
    }
    if audit.sampling_algorithm_id != RLA_SAMPLING_ALGORITHM_ID
        || audit.margin.is_none()
        || audit.stopping_rule_id.as_deref() != Some("comparison-margin-threshold-v1")
        || audit.declared_risk_ppm.is_none()
        || audit.declared_status.is_none()
    {
        return Err(RcountCoreError::MissingColoradoRlaComparisonFields {
            audit_id: audit.audit_id.clone(),
        });
    }
    Ok(())
}

fn verify_california_rla_adapter(audit: &RiskLimitAudit) -> Result<(), RcountCoreError> {
    let Some(ballot_manifest_format_id) = audit.ballot_manifest_format_id.as_deref() else {
        return Err(RcountCoreError::MissingCaliforniaRlaPublicToolFields {
            audit_id: audit.audit_id.clone(),
        });
    };
    if ballot_manifest_format_id != CALIFORNIA_BALLOT_MANIFEST_FORMAT_ID {
        return Err(RcountCoreError::InvalidCaliforniaRlaManifestFormat {
            audit_id: audit.audit_id.clone(),
            ballot_manifest_format_id: ballot_manifest_format_id.to_string(),
        });
    }
    if audit.audit_software_id.as_deref().is_none_or(str::is_empty)
        || audit
            .audit_software_source_url
            .as_deref()
            .is_none_or(str::is_empty)
        || audit.margin.is_none()
        || audit.declared_status.is_none()
    {
        return Err(RcountCoreError::MissingCaliforniaRlaPublicToolFields {
            audit_id: audit.audit_id.clone(),
        });
    }
    let source_url = audit.audit_software_source_url.as_deref().unwrap();
    if !(source_url.starts_with("https://") || source_url.starts_with("http://")) {
        return Err(RcountCoreError::InvalidRlaSoftwareSourceUrl {
            audit_id: audit.audit_id.clone(),
            source_url: source_url.to_string(),
        });
    }
    Ok(())
}

pub fn verify_manual_audits(package: &RcountPackage) -> Result<Vec<EquationPass>, RcountCoreError> {
    let mut seen = BTreeSet::new();
    let mut passes = Vec::new();
    for audit in &package.manual_audits {
        if !seen.insert(audit.audit_id.as_str()) {
            return Err(RcountCoreError::DuplicateManualAuditId {
                audit_id: audit.audit_id.clone(),
            });
        }
        let summary = package
            .summaries
            .iter()
            .find(|summary| {
                summary.contest_id == audit.contest_id
                    && summary.reporting_unit_id == audit.reporting_unit_id
                    && summary.status == CountStatus::Canvassed
                    && summary.batch_id.is_none()
            })
            .ok_or_else(|| RcountCoreError::MissingManualAuditSummary {
                audit_id: audit.audit_id.clone(),
                contest_id: audit.contest_id.clone(),
                reporting_unit_id: audit.reporting_unit_id.clone(),
            })?;
        let summary_totals: BTreeMap<&str, i64> = summary
            .totals
            .iter()
            .map(|total| (total.selection_id.as_str(), total.votes))
            .collect();
        let machine_totals: BTreeMap<&str, i64> = audit
            .machine_totals
            .iter()
            .map(|total| (total.selection_id.as_str(), total.votes))
            .collect();
        for (selection_id, summary_votes) in &summary_totals {
            let declared = machine_totals.get(selection_id).copied().ok_or_else(|| {
                RcountCoreError::ManualAuditMachineTotalMismatch {
                    audit_id: audit.audit_id.clone(),
                    selection_id: (*selection_id).to_string(),
                    declared: 0,
                    summary: *summary_votes,
                }
            })?;
            if declared != *summary_votes {
                return Err(RcountCoreError::ManualAuditMachineTotalMismatch {
                    audit_id: audit.audit_id.clone(),
                    selection_id: (*selection_id).to_string(),
                    declared,
                    summary: *summary_votes,
                });
            }
        }
        let hand_totals: BTreeMap<&str, i64> = audit
            .hand_totals
            .iter()
            .map(|total| (total.selection_id.as_str(), total.votes))
            .collect();
        let computed = if summary_totals.iter().all(|(selection_id, machine_votes)| {
            hand_totals.get(selection_id).is_some_and(|hand_votes| {
                (*hand_votes - *machine_votes).abs() <= audit.tolerance_votes
            })
        }) {
            ManualAuditStatus::Pass
        } else {
            ManualAuditStatus::Escalate
        };
        if audit.declared_status != computed {
            return Err(RcountCoreError::ManualAuditStatusMismatch {
                audit_id: audit.audit_id.clone(),
                declared: audit.declared_status,
                computed,
            });
        }
        passes.push(EquationPass {
            equation_id: "manual_audit_reconciliation".to_string(),
            contest_id: audit.contest_id.clone(),
            reporting_unit_id: audit.reporting_unit_id.clone(),
        });
    }
    Ok(passes)
}

pub fn verify_batch_comparison_audits(
    package: &RcountPackage,
) -> Result<Vec<EquationPass>, RcountCoreError> {
    let mut seen = BTreeSet::new();
    let mut passes = Vec::new();
    for audit in &package.batch_comparison_audits {
        if !seen.insert(audit.audit_id.as_str()) {
            return Err(RcountCoreError::DuplicateBatchComparisonAuditId {
                audit_id: audit.audit_id.clone(),
            });
        }
        let batch = package
            .batches
            .iter()
            .find(|batch| batch.batch_id == audit.batch_id)
            .ok_or_else(|| RcountCoreError::MissingBatchComparisonBatch {
                audit_id: audit.audit_id.clone(),
                batch_id: audit.batch_id.clone(),
            })?;
        if audit.declared_batch_ballots != batch.counted_ballots {
            return Err(RcountCoreError::BatchComparisonBatchSizeMismatch {
                audit_id: audit.audit_id.clone(),
                batch_id: audit.batch_id.clone(),
                declared: audit.declared_batch_ballots,
                manifest: batch.counted_ballots,
            });
        }
        let summary = package
            .summaries
            .iter()
            .find(|summary| {
                summary.contest_id == audit.contest_id
                    && summary.batch_id.as_deref() == Some(audit.batch_id.as_str())
                    && summary.status == CountStatus::Canvassed
            })
            .ok_or_else(|| RcountCoreError::MissingBatchComparisonSummary {
                audit_id: audit.audit_id.clone(),
                contest_id: audit.contest_id.clone(),
                batch_id: audit.batch_id.clone(),
            })?;
        let summary_totals = totals_by_selection(&summary.totals);
        let reported_totals = totals_by_selection(&audit.reported_totals);
        check_reported_batch_total(
            &audit.audit_id,
            &audit.winner_selection_id,
            &summary_totals,
            &reported_totals,
        )?;
        check_reported_batch_total(
            &audit.audit_id,
            &audit.loser_selection_id,
            &summary_totals,
            &reported_totals,
        )?;

        let hand_totals = totals_by_selection(&audit.hand_totals);
        let reported_winner = required_total(&audit.winner_selection_id, &reported_totals);
        let reported_loser = required_total(&audit.loser_selection_id, &reported_totals);
        let hand_winner =
            required_hand_total(&audit.audit_id, &audit.winner_selection_id, &hand_totals)?;
        let hand_loser =
            required_hand_total(&audit.audit_id, &audit.loser_selection_id, &hand_totals)?;
        let reported_margin = reported_winner - reported_loser;
        let hand_margin = hand_winner - hand_loser;
        let overstatement = reported_margin - hand_margin;
        if audit.declared_reported_margin != reported_margin {
            return Err(RcountCoreError::BatchComparisonReportedMarginMismatch {
                audit_id: audit.audit_id.clone(),
                declared: audit.declared_reported_margin,
                computed: reported_margin,
            });
        }
        if audit.declared_hand_margin != hand_margin {
            return Err(RcountCoreError::BatchComparisonHandMarginMismatch {
                audit_id: audit.audit_id.clone(),
                declared: audit.declared_hand_margin,
                computed: hand_margin,
            });
        }
        if audit.declared_overstatement != overstatement {
            return Err(RcountCoreError::BatchComparisonOverstatementMismatch {
                audit_id: audit.audit_id.clone(),
                declared: audit.declared_overstatement,
                computed: overstatement,
            });
        }
        passes.push(EquationPass {
            equation_id: "batch_comparison_overstatement".to_string(),
            contest_id: audit.contest_id.clone(),
            reporting_unit_id: audit.batch_id.clone(),
        });
    }
    Ok(passes)
}

fn check_reported_batch_total(
    audit_id: &str,
    selection_id: &str,
    summary_totals: &BTreeMap<&str, i64>,
    reported_totals: &BTreeMap<&str, i64>,
) -> Result<(), RcountCoreError> {
    let summary = required_total(selection_id, summary_totals);
    let declared = required_total(selection_id, reported_totals);
    if declared != summary {
        return Err(RcountCoreError::BatchComparisonReportedTotalMismatch {
            audit_id: audit_id.to_string(),
            selection_id: selection_id.to_string(),
            declared,
            summary,
        });
    }
    Ok(())
}

fn totals_by_selection(totals: &[SelectionTotal]) -> BTreeMap<&str, i64> {
    totals
        .iter()
        .map(|total| (total.selection_id.as_str(), total.votes))
        .collect()
}

fn required_total(selection_id: &str, totals: &BTreeMap<&str, i64>) -> i64 {
    totals.get(selection_id).copied().unwrap_or(0)
}

fn required_hand_total(
    audit_id: &str,
    selection_id: &str,
    totals: &BTreeMap<&str, i64>,
) -> Result<i64, RcountCoreError> {
    totals.get(selection_id).copied().ok_or_else(|| {
        RcountCoreError::MissingBatchComparisonHandTally {
            audit_id: audit_id.to_string(),
            selection_id: selection_id.to_string(),
        }
    })
}

fn verify_declared_rla_discrepancies(
    audit: &RiskLimitAudit,
    computed: &[RlaDiscrepancy],
) -> Result<(), RcountCoreError> {
    if audit.discrepancies.is_empty() && computed.is_empty() {
        return Ok(());
    }
    let mut declared = audit.discrepancies.clone();
    declared.sort_by_key(|item| (item.draw_index, item.cvr_id.clone(), item.kind));
    let mut computed = computed.to_vec();
    computed.sort_by_key(|item| (item.draw_index, item.cvr_id.clone(), item.kind));
    if declared.len() != computed.len() {
        return Err(RcountCoreError::RlaDiscrepancyCountMismatch {
            audit_id: audit.audit_id.clone(),
            declared: declared.len(),
            computed: computed.len(),
        });
    }
    for (declared, computed) in declared.iter().zip(computed.iter()) {
        if declared.draw_index != computed.draw_index
            || declared.cvr_id != computed.cvr_id
            || declared.kind != computed.kind
        {
            return Err(RcountCoreError::RlaDiscrepancyMismatch {
                audit_id: audit.audit_id.clone(),
                draw_index: computed.draw_index,
                declared: declared.kind,
                computed: computed.kind,
            });
        }
    }
    Ok(())
}

fn classify_rla_discrepancy(
    observation: &RlaSampleObservation,
    cvr: &CvrContestRecord,
) -> Option<RlaDiscrepancyKind> {
    let mut observed = observation.observed_selection_ids.clone();
    observed.sort();
    let mut expected = cvr.selection_ids.clone();
    expected.sort();
    let selection_mismatch = observed != expected;
    let residual_mismatch = observation.undervote != cvr.undervote
        || observation.overvote != cvr.overvote
        || observation.blank_contest != cvr.blank_contest;
    match (selection_mismatch, residual_mismatch) {
        (true, true) => Some(RlaDiscrepancyKind::SelectionAndResidualMismatch),
        (true, false) => Some(RlaDiscrepancyKind::SelectionMismatch),
        (false, true) => Some(RlaDiscrepancyKind::ResidualMismatch),
        (false, false) => None,
    }
}

fn comparison_margin_risk_ppm(audit: &RiskLimitAudit) -> u32 {
    let margin = audit
        .margin
        .as_ref()
        .expect("comparison margin verifier requires margin metadata");
    let sample_margin_product =
        (audit.sample_size as u128).saturating_mul(margin.reported_margin.max(1) as u128);
    let denominator = sample_margin_product.max(1);
    let base = (1_000_000u128 + denominator - 1) / denominator;
    let discrepancy_penalty = (audit.discrepancies.len() as u128).saturating_mul(250_000);
    base.saturating_add(discrepancy_penalty).min(1_000_000) as u32
}

fn rla_population(package: &RcountPackage, contest_id: &str) -> Vec<String> {
    let mut population: Vec<String> = package
        .cvr
        .iter()
        .filter(|row| row.contest_id == contest_id)
        .map(|row| row.cvr_id.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    population.sort();
    population
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct CvrAggregateKey {
    contest_id: String,
    reporting_unit_id: String,
    batch_id: Option<String>,
    status: CountStatus,
}

impl From<&CvrContestRecord> for CvrAggregateKey {
    fn from(row: &CvrContestRecord) -> Self {
        Self {
            contest_id: row.contest_id.clone(),
            reporting_unit_id: row.reporting_unit_id.clone(),
            batch_id: row.batch_id.clone(),
            status: row.status,
        }
    }
}

#[derive(Debug, Default)]
struct CvrAggregate {
    selection_votes: BTreeMap<String, i64>,
    undervotes: i64,
    overvotes: i64,
    blank_contests: i64,
    counted_ballots: i64,
}

pub fn synthetic_summary_basic_package() -> RcountPackage {
    let contest = Contest {
        contest_id: "syn-2024-mayor".to_string(),
        title: "Synthetic Mayor".to_string(),
        vote_for: 1,
        selections: vec![
            Selection {
                selection_id: "cand-a".to_string(),
                kind: SelectionKind::Candidate,
                label: "Candidate A".to_string(),
            },
            Selection {
                selection_id: "cand-b".to_string(),
                kind: SelectionKind::Candidate,
                label: "Candidate B".to_string(),
            },
            Selection {
                selection_id: "write-in".to_string(),
                kind: SelectionKind::WriteInBucket,
                label: "Write-in".to_string(),
            },
        ],
    };
    let reporting_units = vec![
        ReportingUnit {
            reporting_unit_id: "syn:precinct:P-001".to_string(),
            kind: ReportingUnitKind::Precinct,
            parent_jurisdiction: "syn-county-1".to_string(),
            source_ids: vec!["P-001".to_string()],
            valid_from: Some("2024-11-05".to_string()),
            valid_to: None,
        },
        ReportingUnit {
            reporting_unit_id: "syn:precinct:P-002".to_string(),
            kind: ReportingUnitKind::Precinct,
            parent_jurisdiction: "syn-county-1".to_string(),
            source_ids: vec!["P-002".to_string()],
            valid_from: Some("2024-11-05".to_string()),
            valid_to: None,
        },
        ReportingUnit {
            reporting_unit_id: "syn:jurisdiction:SYN".to_string(),
            kind: ReportingUnitKind::JurisdictionTotal,
            parent_jurisdiction: "syn".to_string(),
            source_ids: vec!["SYN".to_string()],
            valid_from: Some("2024-11-05".to_string()),
            valid_to: None,
        },
    ];
    let summaries = vec![
        summary("syn:precinct:P-001", 40, 35, 1, 3, 1, 0),
        summary("syn:precinct:P-002", 25, 30, 0, 4, 0, 1),
        summary("syn:jurisdiction:SYN", 65, 65, 1, 7, 1, 1),
    ];
    RcountPackage {
        rcount_version: RCOUNT_VERSION.to_string(),
        contests: vec![contest],
        reporting_units,
        batches: vec![],
        lineage: vec![],
        rhist_refs: vec![],
        rctx_refs: vec![],
        inclusion_proofs: vec![],
        cvr: vec![],
        audit_algorithm_runs: vec![],
        rla_audits: vec![],
        manual_audits: vec![],
        batch_comparison_audits: vec![],
        summaries,
        status_events: vec![],
    }
}

pub fn synthetic_summary_basic_package_with_base_references() -> RcountPackage {
    let mut package = synthetic_summary_basic_package();
    package.rctx_refs = vec![RctxReference {
        reference_id: "rctx:syn-l0-shared-context".to_string(),
        context_hash: SYN_RCTX_L0_CONTEXT_HASH.to_string(),
        context_path: Some("docs/fixtures/rctx/l0-shared-context".to_string()),
        crosswalk_hash: Some(SYN_RCTX_L0_CROSSWALK_HASH.to_string()),
        crosswalk_path: Some("docs/fixtures/rctx/l0-shared-context/units/crosswalks.ndjson".to_string()),
        role: "aggregation-crosswalk".to_string(),
        note: Some(format!(
            "References RCTX fixture package {SYN_RCTX_L0_PACKAGE_HASH} by stable context and crosswalk hashes."
        )),
    }];
    package.rhist_refs = vec![RhistReference {
        reference_id: "rhist:syn-l2-three-cycle".to_string(),
        package_hash: SYN_RHIST_L2_PACKAGE_HASH.to_string(),
        package_path: Some("docs/fixtures/rhist/l2-three-cycle".to_string()),
        cycle_ids: vec![
            "syn-2024-general".to_string(),
            "syn-2026-general".to_string(),
            "syn-2028-general".to_string(),
        ],
        role: "unit-lineage".to_string(),
        note: Some("References RHIST rename/split/merge fixture by package hash.".to_string()),
    }];
    package
}

pub fn synthetic_canvass_correction_package() -> RcountPackage {
    let mut package = synthetic_summary_basic_package();
    let unofficial = vec![
        summary_with_status(
            "syn:precinct:P-001",
            CountStatus::Unofficial,
            40,
            34,
            1,
            3,
            1,
            0,
        ),
        summary_with_status(
            "syn:precinct:P-002",
            CountStatus::Unofficial,
            25,
            30,
            0,
            4,
            0,
            1,
        ),
        summary_with_status(
            "syn:jurisdiction:SYN",
            CountStatus::Unofficial,
            65,
            64,
            1,
            7,
            1,
            1,
        ),
    ];
    let canvassed = vec![
        summary_with_status(
            "syn:precinct:P-001",
            CountStatus::Canvassed,
            40,
            35,
            1,
            3,
            1,
            0,
        ),
        summary_with_status(
            "syn:precinct:P-002",
            CountStatus::Canvassed,
            25,
            30,
            0,
            4,
            0,
            1,
        ),
        summary_with_status(
            "syn:jurisdiction:SYN",
            CountStatus::Canvassed,
            65,
            65,
            1,
            7,
            1,
            1,
        ),
    ];
    package.summaries = unofficial.into_iter().chain(canvassed).collect();
    package.status_events = vec![
        StatusEvent {
            event_id: "event-0001".to_string(),
            event_type: StatusEventType::InitialUnofficialReport,
            status_before: CountStatus::Unofficial,
            status_after: CountStatus::Unofficial,
            effective_at: "2024-11-05T23:00:00Z".to_string(),
            authority: "SYN County Election Office".to_string(),
            source_refs: vec!["source:unofficial-election-night-export".to_string()],
            explanation: "Election-night unofficial report loaded from the first public export.".to_string(),
        },
        StatusEvent {
            event_id: "event-0002".to_string(),
            event_type: StatusEventType::Correction,
            status_before: CountStatus::Unofficial,
            status_after: CountStatus::Canvassed,
            effective_at: "2024-11-12T18:22:00Z".to_string(),
            authority: "SYN County Canvassing Board".to_string(),
            source_refs: vec!["source:canvass-minutes-2024-11-12".to_string()],
            explanation: "Canvass correction added one Candidate B vote in P-001 after write-in adjudication review.".to_string(),
        },
    ];
    package
}

pub fn synthetic_bad_selection_sum_package() -> RcountPackage {
    let mut package = synthetic_summary_basic_package();
    package.summaries[0].counted_ballots += 1;
    package
}

pub fn synthetic_mail_batch_added_package() -> RcountPackage {
    let mut package = synthetic_summary_basic_package();
    package.batches = vec![
        BatchManifest {
            batch_id: "batch:P-001:election-day".to_string(),
            reporting_unit_id: "syn:precinct:P-001".to_string(),
            kind: BatchKind::ElectionDay,
            status: CountStatus::Canvassed,
            accepted_ballots: 70,
            counted_ballots: 70,
            rejected_ballots: 0,
            source_refs: vec!["source:synthetic-summary-export".to_string()],
        },
        BatchManifest {
            batch_id: "batch:P-001:late-mail".to_string(),
            reporting_unit_id: "syn:precinct:P-001".to_string(),
            kind: BatchKind::Mail,
            status: CountStatus::Canvassed,
            accepted_ballots: 10,
            counted_ballots: 10,
            rejected_ballots: 0,
            source_refs: vec!["source:synthetic-summary-export".to_string()],
        },
        BatchManifest {
            batch_id: "batch:P-002:election-day".to_string(),
            reporting_unit_id: "syn:precinct:P-002".to_string(),
            kind: BatchKind::ElectionDay,
            status: CountStatus::Canvassed,
            accepted_ballots: 60,
            counted_ballots: 60,
            rejected_ballots: 0,
            source_refs: vec!["source:synthetic-summary-export".to_string()],
        },
    ];
    package.summaries = vec![
        summary_with_status_and_batch(
            "syn:precinct:P-001",
            CountStatus::Canvassed,
            Some("batch:P-001:election-day"),
            35,
            30,
            1,
            3,
            1,
            0,
        ),
        summary_with_status_and_batch(
            "syn:precinct:P-001",
            CountStatus::Canvassed,
            Some("batch:P-001:late-mail"),
            5,
            5,
            0,
            0,
            0,
            0,
        ),
        summary_with_status_and_batch(
            "syn:precinct:P-002",
            CountStatus::Canvassed,
            Some("batch:P-002:election-day"),
            25,
            30,
            0,
            4,
            0,
            1,
        ),
        summary("syn:jurisdiction:SYN", 65, 65, 1, 7, 1, 1),
    ];
    package.status_events = vec![StatusEvent {
        event_id: "event-0003".to_string(),
        event_type: StatusEventType::LateMailBatchAdded,
        status_before: CountStatus::Unofficial,
        status_after: CountStatus::Canvassed,
        effective_at: "2024-11-08T17:00:00Z".to_string(),
        authority: "SYN County Election Office".to_string(),
        source_refs: vec!["source:synthetic-summary-export".to_string()],
        explanation: "Late-arriving mail batch for P-001 was accepted before canvass.".to_string(),
    }];
    package
}

pub fn synthetic_missing_batch_package() -> RcountPackage {
    let mut package = synthetic_mail_batch_added_package();
    package
        .batches
        .retain(|batch| batch.batch_id != "batch:P-001:late-mail");
    package
}

pub fn synthetic_batch_comparison_package() -> RcountPackage {
    let mut package = synthetic_mail_batch_added_package();
    package.batch_comparison_audits = vec![BatchComparisonAudit {
        audit_id: "batch-comparison:P-001-election-day".to_string(),
        contest_id: "syn-2024-mayor".to_string(),
        batch_id: "batch:P-001:election-day".to_string(),
        declared_batch_ballots: 70,
        winner_selection_id: "cand-a".to_string(),
        loser_selection_id: "cand-b".to_string(),
        reported_totals: vec![
            SelectionTotal {
                selection_id: "cand-a".to_string(),
                votes: 35,
            },
            SelectionTotal {
                selection_id: "cand-b".to_string(),
                votes: 30,
            },
        ],
        hand_totals: vec![
            SelectionTotal {
                selection_id: "cand-a".to_string(),
                votes: 34,
            },
            SelectionTotal {
                selection_id: "cand-b".to_string(),
                votes: 31,
            },
        ],
        declared_reported_margin: 5,
        declared_hand_margin: 3,
        declared_overstatement: 2,
        source_refs: vec!["source:synthetic-batch-hand-tally".to_string()],
    }];
    package
}

pub fn synthetic_batch_comparison_algorithm_package() -> RcountPackage {
    let mut package = synthetic_batch_comparison_package();
    package.audit_algorithm_runs = vec![derive_batch_comparison_algorithm_run(
        &package,
        "audit-run:batch-comparison-taint-linkage",
        "syn-2024-mayor",
        300_000,
        &["batch:P-001:election-day".to_string()],
        AuditAlgorithmDecision::Continue,
    )
    .expect("synthetic batch comparison algorithm run must derive")];
    package
}

pub fn synthetic_kaplan_markov_macro_package() -> RcountPackage {
    let mut package = synthetic_summary_basic_package();
    package.audit_algorithm_runs = vec![AuditAlgorithmRun {
        run_id: "audit-run:kaplan-markov-macro-pass".to_string(),
        contest_id: "syn-2024-mayor".to_string(),
        method_id: KAPLAN_MARKOV_COMPARISON_METHOD_ID.to_string(),
        sampling_mode: AuditSamplingMode::WithoutReplacement,
        rcv_elimination_order: Vec::new(),
        risk_limit_ppm: Some(500_000),
        reported_winner_votes: Some(65),
        reported_loser_votes: Some(65),
        macro_ballot_count: Some(100),
        macro_reported_margin: Some(10),
        macro_gamma: Some(RationalValue {
            numerator: 11,
            denominator: 10,
        }),
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
            assorter_id: "macro-overstatement-category-v1".to_string(),
            assorter_upper_bound: RationalValue {
                numerator: 2,
                denominator: 1,
            },
            winner_selection_id: Some("cand-a".to_string()),
            loser_selection_id: Some("cand-b".to_string()),
        }],
        sample_steps: (0..16)
            .map(|step_index| AuditSampleStep {
                step_index,
                round_index: None,
                assertion_id: "assertion:cand-a-over-cand-b".to_string(),
                sample_unit_id: format!("ballot:macro:{step_index}"),
                assorter_value: RationalValue {
                    numerator: 0,
                    denominator: 1,
                },
                bet: None,
                statistic: None,
                p_value_ppm: None,
                ranked_choices: Vec::new(),
                source_refs: vec![format!("source:macro-ballot:{step_index}")],
            })
            .collect(),
        decision: AuditAlgorithmDecision::Pass,
        source_refs: vec!["source:synthetic-macro-comparison-audit".to_string()],
    }];
    package
}

pub fn synthetic_minerva_round_one_package() -> RcountPackage {
    let mut package = synthetic_summary_basic_package();
    package.audit_algorithm_runs = vec![AuditAlgorithmRun {
        run_id: "audit-run:minerva-round-one-pass".to_string(),
        contest_id: "syn-2024-mayor".to_string(),
        method_id: MINERVA_BALLOT_POLLING_METHOD_ID.to_string(),
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
                sample_unit_id: format!("ballot:minerva:{step_index}"),
                assorter_value: RationalValue {
                    numerator: 1,
                    denominator: 1,
                },
                bet: None,
                statistic: None,
                p_value_ppm: None,
                ranked_choices: Vec::new(),
                source_refs: vec![format!("source:minerva-ballot:{step_index}")],
            })
            .collect(),
        decision: AuditAlgorithmDecision::Pass,
        source_refs: vec!["source:synthetic-minerva-round-one-audit".to_string()],
    }];
    package
}

pub fn synthetic_minerva_multi_round_package() -> RcountPackage {
    let mut package = synthetic_minerva_round_one_package();
    package.audit_algorithm_runs[0].run_id = "audit-run:minerva-multi-round-pass".to_string();
    for step in &mut package.audit_algorithm_runs[0].sample_steps {
        step.round_index = if step.step_index < 5 {
            Some(0)
        } else {
            Some(1)
        };
    }
    package
}

pub fn synthetic_athena_boundary_package() -> RcountPackage {
    let mut package = synthetic_minerva_multi_round_package();
    let run = &mut package.audit_algorithm_runs[0];
    run.run_id = "audit-run:athena-boundary".to_string();
    run.method_id = ATHENA_BALLOT_POLLING_METHOD_ID.to_string();
    run.decision = AuditAlgorithmDecision::Boundary;
    run.source_refs = vec!["source:synthetic-athena-boundary-audit".to_string()];
    package
}

pub fn synthetic_stratified_hybrid_package() -> RcountPackage {
    let mut package = synthetic_batch_comparison_algorithm_package();
    let minerva_run = synthetic_minerva_multi_round_package()
        .audit_algorithm_runs
        .into_iter()
        .next()
        .expect("synthetic Minerva package must contain one algorithm run");
    let batch_run = package.audit_algorithm_runs[0].clone();
    package.audit_algorithm_runs.push(minerva_run);
    package.audit_algorithm_runs.push(AuditAlgorithmRun {
        run_id: "audit-run:stratified-hybrid-boundary".to_string(),
        contest_id: "syn-2024-mayor".to_string(),
        method_id: STRATIFIED_HYBRID_RLA_METHOD_ID.to_string(),
        sampling_mode: AuditSamplingMode::BoundaryOnly,
        rcv_elimination_order: Vec::new(),
        risk_limit_ppm: Some(100_000),
        reported_winner_votes: None,
        reported_loser_votes: None,
        macro_ballot_count: None,
        macro_reported_margin: None,
        macro_gamma: None,
        combining_rule_id: Some("suite-nuisance-boundary-v1".to_string()),
        nuisance_parameter: Some(RationalValue {
            numerator: 1,
            denominator: 2,
        }),
        bayesian_prior_id: None,
        bayesian_likelihood_id: None,
        posterior_winner_probability_ppm: None,
        posterior_risk_ppm: None,
        simulation_seed: None,
        posterior_draws: None,
        calibrated_risk_limit_ppm: None,
        strata: vec![
            AuditStratum {
                stratum_id: "stratum:batch-comparison".to_string(),
                method_id: batch_run.method_id,
                component_run_id: batch_run.run_id,
                ballot_count: Some(80),
                allocation_ppm: Some(500_000),
                source_refs: vec!["source:synthetic-batch-hand-tally".to_string()],
            },
            AuditStratum {
                stratum_id: "stratum:ballot-polling".to_string(),
                method_id: MINERVA_BALLOT_POLLING_METHOD_ID.to_string(),
                component_run_id: "audit-run:minerva-multi-round-pass".to_string(),
                ballot_count: Some(60),
                allocation_ppm: Some(500_000),
                source_refs: vec!["source:synthetic-minerva-round-one-audit".to_string()],
            },
        ],
        assertions: Vec::new(),
        sample_steps: Vec::new(),
        decision: AuditAlgorithmDecision::Boundary,
        source_refs: vec![
            "audit-run:batch-comparison-taint-linkage".to_string(),
            "audit-run:minerva-multi-round-pass".to_string(),
        ],
    });
    package
}

pub fn synthetic_bad_stratified_hybrid_package() -> RcountPackage {
    let mut package = synthetic_stratified_hybrid_package();
    package.audit_algorithm_runs[2].strata[1].component_run_id =
        "audit-run:missing-stratum".to_string();
    package
}

pub fn synthetic_flattened_stratified_hybrid_package() -> RcountPackage {
    let mut package = synthetic_stratified_hybrid_package();
    package.audit_algorithm_runs[2].strata.truncate(1);
    package.audit_algorithm_runs[2].strata[0].allocation_ppm = Some(1_000_000);
    package
}

pub fn synthetic_raire_boundary_package() -> RcountPackage {
    synthetic_ranked_choice_boundary_package(
        RAIRE_IRV_METHOD_ID,
        "audit-run:raire-irv-boundary",
        "raire-neb-not-eliminated-before-v1",
    )
}

pub fn synthetic_awaire_boundary_package() -> RcountPackage {
    synthetic_ranked_choice_boundary_package(
        AWAIRE_IRV_METHOD_ID,
        "audit-run:awaire-irv-boundary",
        "awaire-adaptive-irv-v1",
    )
}

pub fn synthetic_bad_raire_boundary_package() -> RcountPackage {
    let mut package = synthetic_raire_boundary_package();
    package.audit_algorithm_runs[0].sample_steps[0]
        .ranked_choices
        .push("cand-a".to_string());
    package
}

pub fn synthetic_bayesian_tabulation_boundary_package() -> RcountPackage {
    let mut package = synthetic_summary_basic_package();
    package.audit_algorithm_runs = vec![AuditAlgorithmRun {
        run_id: "audit-run:bayesian-tabulation-boundary".to_string(),
        contest_id: "syn-2024-mayor".to_string(),
        method_id: BAYESIAN_TABULATION_AUDIT_METHOD_ID.to_string(),
        sampling_mode: AuditSamplingMode::BoundaryOnly,
        rcv_elimination_order: Vec::new(),
        risk_limit_ppm: None,
        reported_winner_votes: None,
        reported_loser_votes: None,
        macro_ballot_count: None,
        macro_reported_margin: None,
        macro_gamma: None,
        combining_rule_id: None,
        nuisance_parameter: None,
        bayesian_prior_id: Some("dirichlet-multinomial-toy-prior-v1".to_string()),
        bayesian_likelihood_id: Some("sample-counts-without-replacement-v1".to_string()),
        posterior_winner_probability_ppm: Some(958_000),
        posterior_risk_ppm: Some(42_000),
        simulation_seed: Some(20_240_513),
        posterior_draws: Some(10_000),
        calibrated_risk_limit_ppm: None,
        strata: Vec::new(),
        assertions: vec![AuditAssertion {
            assertion_id: "assertion:bayesian-cand-a-outcome".to_string(),
            kind: AuditAssertionKind::BayesianOutcome,
            assorter_id: "bayesian-posterior-winner-probability-v1".to_string(),
            assorter_upper_bound: RationalValue {
                numerator: 1,
                denominator: 1,
            },
            winner_selection_id: Some("cand-a".to_string()),
            loser_selection_id: Some("cand-b".to_string()),
        }],
        sample_steps: Vec::new(),
        decision: AuditAlgorithmDecision::Boundary,
        source_refs: vec!["source:synthetic-bayesian-tabulation-audit".to_string()],
    }];
    package
}

pub fn synthetic_bad_bayesian_tabulation_boundary_package() -> RcountPackage {
    let mut package = synthetic_bayesian_tabulation_boundary_package();
    package.audit_algorithm_runs[0].posterior_risk_ppm = Some(1_000_001);
    package
}

fn synthetic_ranked_choice_boundary_package(
    method_id: &str,
    run_id: &str,
    assorter_id: &str,
) -> RcountPackage {
    let mut package = synthetic_summary_basic_package();
    package.audit_algorithm_runs = vec![AuditAlgorithmRun {
        run_id: run_id.to_string(),
        contest_id: "syn-2024-mayor".to_string(),
        method_id: method_id.to_string(),
        sampling_mode: AuditSamplingMode::WithoutReplacement,
        rcv_elimination_order: vec![
            "cand-c".to_string(),
            "cand-b".to_string(),
            "cand-a".to_string(),
        ],
        risk_limit_ppm: Some(100_000),
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
            assertion_id: "assertion:irv-cand-a-over-cand-b".to_string(),
            kind: AuditAssertionKind::RankedChoiceAssertion,
            assorter_id: assorter_id.to_string(),
            assorter_upper_bound: RationalValue {
                numerator: 1,
                denominator: 1,
            },
            winner_selection_id: Some("cand-a".to_string()),
            loser_selection_id: Some("cand-b".to_string()),
        }],
        sample_steps: vec![
            AuditSampleStep {
                step_index: 0,
                round_index: None,
                assertion_id: "assertion:irv-cand-a-over-cand-b".to_string(),
                sample_unit_id: "ranked-ballot:0".to_string(),
                assorter_value: RationalValue {
                    numerator: 1,
                    denominator: 1,
                },
                bet: None,
                statistic: None,
                p_value_ppm: None,
                ranked_choices: vec![
                    "cand-a".to_string(),
                    "cand-b".to_string(),
                    "cand-c".to_string(),
                ],
                source_refs: vec!["source:synthetic-ranked-cvr:0".to_string()],
            },
            AuditSampleStep {
                step_index: 1,
                round_index: None,
                assertion_id: "assertion:irv-cand-a-over-cand-b".to_string(),
                sample_unit_id: "ranked-ballot:1".to_string(),
                assorter_value: RationalValue {
                    numerator: 0,
                    denominator: 1,
                },
                bet: None,
                statistic: None,
                p_value_ppm: None,
                ranked_choices: vec![
                    "cand-b".to_string(),
                    "cand-a".to_string(),
                    "cand-c".to_string(),
                ],
                source_refs: vec!["source:synthetic-ranked-cvr:1".to_string()],
            },
        ],
        decision: AuditAlgorithmDecision::Boundary,
        source_refs: vec!["source:synthetic-ranked-choice-audit".to_string()],
    }];
    package
}

pub fn synthetic_bad_batch_comparison_algorithm_package() -> RcountPackage {
    let mut package = synthetic_batch_comparison_algorithm_package();
    package.audit_algorithm_runs[0].sample_steps[0].assorter_value = RationalValue {
        numerator: 1,
        denominator: 5,
    };
    package
}

pub fn synthetic_bad_batch_comparison_package() -> RcountPackage {
    let mut package = synthetic_batch_comparison_package();
    package.batch_comparison_audits[0].declared_overstatement = 0;
    package
}

pub fn synthetic_missing_hand_tally_batch_comparison_package() -> RcountPackage {
    let mut package = synthetic_batch_comparison_package();
    package.batch_comparison_audits[0]
        .hand_totals
        .retain(|total| total.selection_id != "cand-b");
    package
}

pub fn synthetic_batch_size_drift_comparison_package() -> RcountPackage {
    let mut package = synthetic_batch_comparison_package();
    package.batch_comparison_audits[0].declared_batch_ballots = 69;
    package
}

pub fn synthetic_precinct_split_lineage_package() -> RcountPackage {
    let mut package = synthetic_summary_basic_package();
    package.reporting_units.extend([
        ReportingUnit {
            reporting_unit_id: "syn:precinct:P-004".to_string(),
            kind: ReportingUnitKind::Precinct,
            parent_jurisdiction: "syn-county-1".to_string(),
            source_ids: vec!["P-004".to_string()],
            valid_from: Some("2024-11-05".to_string()),
            valid_to: Some("2028-11-07".to_string()),
        },
        ReportingUnit {
            reporting_unit_id: "syn:precinct:P-004A".to_string(),
            kind: ReportingUnitKind::SplitPrecinct,
            parent_jurisdiction: "syn-county-1".to_string(),
            source_ids: vec!["P-004A".to_string()],
            valid_from: Some("2028-11-07".to_string()),
            valid_to: None,
        },
        ReportingUnit {
            reporting_unit_id: "syn:precinct:P-004B".to_string(),
            kind: ReportingUnitKind::SplitPrecinct,
            parent_jurisdiction: "syn-county-1".to_string(),
            source_ids: vec!["P-004B".to_string()],
            valid_from: Some("2028-11-07".to_string()),
            valid_to: None,
        },
        ReportingUnit {
            reporting_unit_id: "syn:precinct:P-007".to_string(),
            kind: ReportingUnitKind::Precinct,
            parent_jurisdiction: "syn-county-1".to_string(),
            source_ids: vec!["P-007".to_string()],
            valid_from: Some("2024-11-05".to_string()),
            valid_to: Some("2028-11-07".to_string()),
        },
        ReportingUnit {
            reporting_unit_id: "syn:precinct:P-008".to_string(),
            kind: ReportingUnitKind::Precinct,
            parent_jurisdiction: "syn-county-1".to_string(),
            source_ids: vec!["P-008".to_string()],
            valid_from: Some("2024-11-05".to_string()),
            valid_to: Some("2028-11-07".to_string()),
        },
        ReportingUnit {
            reporting_unit_id: "syn:precinct:P-078".to_string(),
            kind: ReportingUnitKind::Precinct,
            parent_jurisdiction: "syn-county-1".to_string(),
            source_ids: vec!["P-078".to_string()],
            valid_from: Some("2028-11-07".to_string()),
            valid_to: None,
        },
    ]);
    package.lineage = vec![
        ReportingUnitLineage {
            lineage_id: "lineage:P-004-split".to_string(),
            kind: LineageKind::Split,
            prior_cycle: "SYN-2024-general".to_string(),
            current_cycle: "SYN-2028-general".to_string(),
            prior_reporting_unit_ids: vec!["syn:precinct:P-004".to_string()],
            current_reporting_unit_ids: vec![
                "syn:precinct:P-004A".to_string(),
                "syn:precinct:P-004B".to_string(),
            ],
            authority: "SYN County Election Office".to_string(),
            explanation: "P-004 split into two precincts after municipal growth.".to_string(),
        },
        ReportingUnitLineage {
            lineage_id: "lineage:P-007-P-008-merge".to_string(),
            kind: LineageKind::Merge,
            prior_cycle: "SYN-2024-general".to_string(),
            current_cycle: "SYN-2028-general".to_string(),
            prior_reporting_unit_ids: vec![
                "syn:precinct:P-007".to_string(),
                "syn:precinct:P-008".to_string(),
            ],
            current_reporting_unit_ids: vec!["syn:precinct:P-078".to_string()],
            authority: "SYN County Election Office".to_string(),
            explanation: "P-007 and P-008 merged into P-078 during precinct consolidation."
                .to_string(),
        },
    ];
    package
}

pub fn synthetic_bad_lineage_package() -> RcountPackage {
    let mut package = synthetic_precinct_split_lineage_package();
    package.lineage[0]
        .current_reporting_unit_ids
        .push("syn:precinct:P-004C".to_string());
    package
}

pub fn synthetic_privacy_inclusion_package() -> RcountPackage {
    let mut package = synthetic_summary_basic_package();
    package.inclusion_proofs = vec![InclusionProof {
        proof_id: "proof:accepted-token-001".to_string(),
        kind: InclusionProofKind::AnonymizedAcceptedBallotToken,
        token_hash: format!("sha256:{}", "a".repeat(64)),
        reporting_unit_id: "syn:precinct:P-001".to_string(),
        candidate_selections: vec![],
        voter_id: None,
        ballot_style: None,
        issued_at: None,
    }];
    package
}

pub fn synthetic_choice_bearing_proof_package() -> RcountPackage {
    let mut package = synthetic_privacy_inclusion_package();
    package.inclusion_proofs[0].candidate_selections = vec!["cand-a".to_string()];
    package
}

pub fn synthetic_soba_observable_ballot_boundary_package() -> RcountPackage {
    let mut package = synthetic_privacy_inclusion_package();
    package.audit_algorithm_runs = vec![AuditAlgorithmRun {
        run_id: "audit-run:soba-observable-ballot-boundary".to_string(),
        contest_id: "syn-2024-mayor".to_string(),
        method_id: SOBA_OBSERVABLE_BALLOT_AUDIT_METHOD_ID.to_string(),
        sampling_mode: AuditSamplingMode::BoundaryOnly,
        rcv_elimination_order: Vec::new(),
        risk_limit_ppm: None,
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
            assertion_id: "assertion:observable-ballot-opening".to_string(),
            kind: AuditAssertionKind::ObservableBallotLinkage,
            assorter_id: "soba-commitment-opening-v1".to_string(),
            assorter_upper_bound: RationalValue {
                numerator: 1,
                denominator: 1,
            },
            winner_selection_id: None,
            loser_selection_id: None,
        }],
        sample_steps: vec![AuditSampleStep {
            step_index: 0,
            round_index: None,
            assertion_id: "assertion:observable-ballot-opening".to_string(),
            sample_unit_id: "proof:accepted-token-001".to_string(),
            assorter_value: RationalValue {
                numerator: 1,
                denominator: 1,
            },
            bet: None,
            statistic: None,
            p_value_ppm: None,
            ranked_choices: Vec::new(),
            source_refs: vec!["source:synthetic-soba-opening".to_string()],
        }],
        decision: AuditAlgorithmDecision::Boundary,
        source_refs: vec!["source:synthetic-soba-observable-ballot-audit".to_string()],
    }];
    package
}

pub fn synthetic_missing_soba_opening_package() -> RcountPackage {
    let mut package = synthetic_soba_observable_ballot_boundary_package();
    package.audit_algorithm_runs[0].sample_steps[0].sample_unit_id =
        "proof:missing-token".to_string();
    package
}

pub fn synthetic_cvr_summary_package() -> RcountPackage {
    let mut package = synthetic_summary_basic_package();
    package.cvr = vec![];
    append_cvr_rows(
        &mut package.cvr,
        "P-001",
        "syn:precinct:P-001",
        40,
        35,
        1,
        3,
        1,
        0,
    );
    append_cvr_rows(
        &mut package.cvr,
        "P-002",
        "syn:precinct:P-002",
        25,
        30,
        0,
        4,
        0,
        1,
    );
    package
}

pub fn synthetic_bad_cvr_summary_package() -> RcountPackage {
    let mut package = synthetic_cvr_summary_package();
    let row = package
        .cvr
        .iter_mut()
        .find(|row| {
            row.reporting_unit_id == "syn:precinct:P-001"
                && row.selection_ids.len() == 1
                && row.selection_ids[0] == "cand-a"
        })
        .expect("synthetic CVR package must contain a Candidate A row");
    row.selection_ids = vec!["cand-b".to_string()];
    package
}

pub fn synthetic_rla_replay_package() -> RcountPackage {
    let mut package = synthetic_cvr_summary_package();
    let manifest_hash = rla_contest_manifest_hash(&package, "syn-2024-mayor")
        .expect("synthetic CVR package must have an RLA population");
    let mut audit = RiskLimitAudit {
        audit_id: "rla:syn-2024-mayor:round-1".to_string(),
        contest_id: "syn-2024-mayor".to_string(),
        jurisdiction_method_id: None,
        ballot_manifest_format_id: None,
        audit_software_id: None,
        audit_software_source_url: None,
        risk_limit_ppm: 50_000,
        public_seed: "31415926535897932384".to_string(),
        sampling_algorithm_id: RLA_SAMPLING_ALGORITHM_ID.to_string(),
        manifest_hash,
        sample_size: 12,
        sample_draws: vec![],
        observations: vec![],
        discrepancies: vec![],
        margin: None,
        stopping_rule_id: None,
        max_discrepancies: None,
        declared_status: None,
        declared_risk_ppm: None,
    };
    audit.sample_draws =
        replay_rla_sample(&package, &audit).expect("synthetic RLA sample must replay");
    package.rla_audits = vec![audit];
    package
}

pub fn synthetic_bad_rla_replay_package() -> RcountPackage {
    let mut package = synthetic_rla_replay_package();
    package.rla_audits[0].sample_draws[0].cvr_id = "cvr:P-999:999".to_string();
    package
}

pub fn synthetic_rla_stopping_package() -> RcountPackage {
    let mut package = synthetic_rla_replay_package();
    let observations = rla_observations_from_sample(&package, &package.rla_audits[0])
        .expect("synthetic RLA observations must match sample");
    let audit = &mut package.rla_audits[0];
    audit.observations = observations;
    audit.stopping_rule_id = Some("zero-discrepancy-threshold-v1".to_string());
    audit.max_discrepancies = Some(0);
    audit.declared_status = Some(RlaStoppingStatus::Pass);
    package
}

pub fn synthetic_rla_margin_package() -> RcountPackage {
    let mut package = synthetic_rla_stopping_package();
    package.rla_audits[0].margin = Some(RlaMarginMetadata {
        winner_selection_id: "cand-a".to_string(),
        loser_selection_id: "write-in".to_string(),
        reported_winner_votes: 65,
        reported_loser_votes: 1,
        reported_margin: 64,
        diluted_margin_denominator: 140,
    });
    package
}

pub fn synthetic_bad_rla_margin_package() -> RcountPackage {
    let mut package = synthetic_rla_margin_package();
    package.rla_audits[0]
        .margin
        .as_mut()
        .expect("synthetic RLA margin package must contain margin")
        .reported_margin += 1;
    package
}

pub fn synthetic_rla_statistical_package() -> RcountPackage {
    let mut package = synthetic_rla_margin_package();
    let risk_ppm = comparison_margin_risk_ppm(&package.rla_audits[0]);
    let audit = &mut package.rla_audits[0];
    audit.stopping_rule_id = Some("comparison-margin-threshold-v1".to_string());
    audit.max_discrepancies = Some(0);
    audit.declared_status = Some(RlaStoppingStatus::Pass);
    audit.declared_risk_ppm = Some(risk_ppm);
    package
}

pub fn synthetic_bad_rla_statistical_package() -> RcountPackage {
    let mut package = synthetic_rla_statistical_package();
    package.rla_audits[0].declared_risk_ppm = Some(
        package.rla_audits[0]
            .declared_risk_ppm
            .expect("synthetic statistical package must contain risk")
            + 1,
    );
    package
}

pub fn synthetic_colorado_rla_package() -> RcountPackage {
    let mut package = synthetic_rla_statistical_package();
    package.rla_audits[0].jurisdiction_method_id = Some(COLORADO_RLA_METHOD_ID.to_string());
    package
}

pub fn synthetic_bad_colorado_rla_package() -> RcountPackage {
    let mut package = synthetic_colorado_rla_package();
    package.rla_audits[0].public_seed = "3141592653589793238X".to_string();
    package.rla_audits[0].sample_draws =
        replay_rla_sample(&package, &package.rla_audits[0]).expect("bad seed still replays");
    package.rla_audits[0].observations =
        rla_observations_from_sample(&package, &package.rla_audits[0])
            .expect("bad Colorado seed package must still have matching observations");
    package
}

pub fn synthetic_california_rla_package() -> RcountPackage {
    let mut package = synthetic_rla_statistical_package();
    let audit = &mut package.rla_audits[0];
    audit.jurisdiction_method_id = Some(CALIFORNIA_RLA_METHOD_ID.to_string());
    audit.ballot_manifest_format_id = Some(CALIFORNIA_BALLOT_MANIFEST_FORMAT_ID.to_string());
    audit.audit_software_id = Some("rcount-open-rla-synthetic-v1".to_string());
    audit.audit_software_source_url = Some(
        "https://github.com/synthetic-election-audit/rcount-open-rla-synthetic-v1".to_string(),
    );
    package
}

pub fn synthetic_bad_california_rla_package() -> RcountPackage {
    let mut package = synthetic_california_rla_package();
    package.rla_audits[0].audit_software_source_url =
        Some("synthetic-election-audit/rcount-open-rla-synthetic-v1".to_string());
    package
}

pub fn synthetic_manual_audit_package() -> RcountPackage {
    let mut package = synthetic_summary_basic_package();
    package.manual_audits = vec![ManualAudit {
        audit_id: "manual-audit:syn-2024-mayor:P-001".to_string(),
        contest_id: "syn-2024-mayor".to_string(),
        reporting_unit_id: "syn:precinct:P-001".to_string(),
        authority: "SYN County Canvassing Board".to_string(),
        audited_batch_ids: vec![],
        tolerance_votes: 0,
        machine_totals: vec![
            SelectionTotal {
                selection_id: "cand-a".to_string(),
                votes: 40,
            },
            SelectionTotal {
                selection_id: "cand-b".to_string(),
                votes: 35,
            },
            SelectionTotal {
                selection_id: "write-in".to_string(),
                votes: 1,
            },
        ],
        hand_totals: vec![
            SelectionTotal {
                selection_id: "cand-a".to_string(),
                votes: 40,
            },
            SelectionTotal {
                selection_id: "cand-b".to_string(),
                votes: 35,
            },
            SelectionTotal {
                selection_id: "write-in".to_string(),
                votes: 1,
            },
        ],
        declared_status: ManualAuditStatus::Pass,
    }];
    package
}

pub fn synthetic_bad_manual_audit_package() -> RcountPackage {
    let mut package = synthetic_manual_audit_package();
    package.manual_audits[0].hand_totals[1].votes += 1;
    package
}

pub fn synthetic_bad_rla_stopping_package() -> RcountPackage {
    let mut package = synthetic_rla_stopping_package();
    package.rla_audits[0].observations[0].observed_selection_ids = vec!["cand-b".to_string()];
    package.rla_audits[0].discrepancies = vec![RlaDiscrepancy {
        draw_index: package.rla_audits[0].sample_draws[0].draw_index,
        cvr_id: package.rla_audits[0].sample_draws[0].cvr_id.clone(),
        kind: RlaDiscrepancyKind::SelectionMismatch,
    }];
    package
}

pub fn synthetic_rla_discrepancy_package() -> RcountPackage {
    let mut package = synthetic_bad_rla_stopping_package();
    package.rla_audits[0].declared_status = Some(RlaStoppingStatus::Escalate);
    package
}

pub fn synthetic_bad_rla_discrepancy_package() -> RcountPackage {
    let mut package = synthetic_rla_discrepancy_package();
    package.rla_audits[0].discrepancies[0].kind = RlaDiscrepancyKind::ResidualMismatch;
    package
}

fn rla_observations_from_sample(
    package: &RcountPackage,
    audit: &RiskLimitAudit,
) -> Result<Vec<RlaSampleObservation>, RcountCoreError> {
    let cvr_by_id: BTreeMap<&str, &CvrContestRecord> = package
        .cvr
        .iter()
        .filter(|row| row.contest_id == audit.contest_id)
        .map(|row| (row.cvr_id.as_str(), row))
        .collect();
    let mut observations = Vec::with_capacity(audit.sample_draws.len());
    for draw in &audit.sample_draws {
        let cvr = cvr_by_id.get(draw.cvr_id.as_str()).ok_or_else(|| {
            RcountCoreError::MissingRlaPopulation {
                audit_id: audit.audit_id.clone(),
                contest_id: audit.contest_id.clone(),
            }
        })?;
        observations.push(RlaSampleObservation {
            draw_index: draw.draw_index,
            cvr_id: draw.cvr_id.clone(),
            observed_selection_ids: cvr.selection_ids.clone(),
            undervote: cvr.undervote,
            overvote: cvr.overvote,
            blank_contest: cvr.blank_contest,
        });
    }
    Ok(observations)
}

fn summary(
    reporting_unit_id: &str,
    cand_a: i64,
    cand_b: i64,
    write_in: i64,
    undervotes: i64,
    overvotes: i64,
    blank_contests: i64,
) -> Summary {
    summary_with_status_and_batch(
        reporting_unit_id,
        CountStatus::Canvassed,
        None,
        cand_a,
        cand_b,
        write_in,
        undervotes,
        overvotes,
        blank_contests,
    )
}

fn summary_with_status(
    reporting_unit_id: &str,
    status: CountStatus,
    cand_a: i64,
    cand_b: i64,
    write_in: i64,
    undervotes: i64,
    overvotes: i64,
    blank_contests: i64,
) -> Summary {
    summary_with_status_and_batch(
        reporting_unit_id,
        status,
        None,
        cand_a,
        cand_b,
        write_in,
        undervotes,
        overvotes,
        blank_contests,
    )
}

fn summary_with_status_and_batch(
    reporting_unit_id: &str,
    status: CountStatus,
    batch_id: Option<&str>,
    cand_a: i64,
    cand_b: i64,
    write_in: i64,
    undervotes: i64,
    overvotes: i64,
    blank_contests: i64,
) -> Summary {
    Summary {
        contest_id: "syn-2024-mayor".to_string(),
        reporting_unit_id: reporting_unit_id.to_string(),
        batch_id: batch_id.map(str::to_string),
        status,
        totals: vec![
            SelectionTotal {
                selection_id: "cand-a".to_string(),
                votes: cand_a,
            },
            SelectionTotal {
                selection_id: "cand-b".to_string(),
                votes: cand_b,
            },
            SelectionTotal {
                selection_id: "write-in".to_string(),
                votes: write_in,
            },
        ],
        undervotes,
        overvotes,
        blank_contests,
        counted_ballots: cand_a + cand_b + write_in + undervotes + overvotes + blank_contests,
    }
}

fn append_cvr_rows(
    rows: &mut Vec<CvrContestRecord>,
    id_prefix: &str,
    reporting_unit_id: &str,
    cand_a: i64,
    cand_b: i64,
    write_in: i64,
    undervotes: i64,
    overvotes: i64,
    blank_contests: i64,
) {
    let mut ordinal = 1usize;
    for (selection_id, count) in [
        ("cand-a", cand_a),
        ("cand-b", cand_b),
        ("write-in", write_in),
    ] {
        for _ in 0..count {
            rows.push(cvr_selection_row(
                id_prefix,
                ordinal,
                reporting_unit_id,
                selection_id,
            ));
            ordinal += 1;
        }
    }
    for _ in 0..undervotes {
        rows.push(cvr_residual_row(
            id_prefix,
            ordinal,
            reporting_unit_id,
            "undervote",
        ));
        ordinal += 1;
    }
    for _ in 0..overvotes {
        rows.push(cvr_residual_row(
            id_prefix,
            ordinal,
            reporting_unit_id,
            "overvote",
        ));
        ordinal += 1;
    }
    for _ in 0..blank_contests {
        rows.push(cvr_residual_row(
            id_prefix,
            ordinal,
            reporting_unit_id,
            "blank",
        ));
        ordinal += 1;
    }
}

fn cvr_selection_row(
    id_prefix: &str,
    ordinal: usize,
    reporting_unit_id: &str,
    selection_id: &str,
) -> CvrContestRecord {
    CvrContestRecord {
        cvr_id: format!("cvr:{id_prefix}:{ordinal:03}"),
        contest_id: "syn-2024-mayor".to_string(),
        reporting_unit_id: reporting_unit_id.to_string(),
        batch_id: None,
        status: CountStatus::Canvassed,
        selection_ids: vec![selection_id.to_string()],
        undervote: false,
        overvote: false,
        blank_contest: false,
        source_refs: vec!["source:synthetic-summary-export".to_string()],
    }
}

fn cvr_residual_row(
    id_prefix: &str,
    ordinal: usize,
    reporting_unit_id: &str,
    residual: &str,
) -> CvrContestRecord {
    CvrContestRecord {
        cvr_id: format!("cvr:{id_prefix}:{ordinal:03}"),
        contest_id: "syn-2024-mayor".to_string(),
        reporting_unit_id: reporting_unit_id.to_string(),
        batch_id: None,
        status: CountStatus::Canvassed,
        selection_ids: vec![],
        undervote: residual == "undervote",
        overvote: residual == "overvote",
        blank_contest: residual == "blank",
        source_refs: vec!["source:synthetic-summary-export".to_string()],
    }
}

fn canonicalize_value(value: &Value) -> Value {
    match value {
        Value::Array(values) => Value::Array(values.iter().map(canonicalize_value).collect()),
        Value::Object(map) => {
            let mut sorted = Map::new();
            let mut keys: Vec<_> = map.keys().collect();
            keys.sort();
            for key in keys {
                sorted.insert(key.clone(), canonicalize_value(&map[key]));
            }
            Value::Object(sorted)
        }
        other => other.clone(),
    }
}

fn validate_contest(contest: &Contest) -> Result<(), RcountCoreError> {
    let mut seen = BTreeSet::new();
    for selection in contest.selections.iter() {
        if !seen.insert(selection.selection_id.as_str()) {
            return Err(RcountCoreError::DuplicateSelectionId {
                contest_id: contest.contest_id.clone(),
                selection_id: selection.selection_id.clone(),
            });
        }
    }
    Ok(())
}

fn ensure_non_negative(value: i64) -> Result<(), RcountCoreError> {
    if value < 0 {
        return Err(RcountCoreError::NegativeCount);
    }
    Ok(())
}

fn is_sha256_hash(value: &str) -> bool {
    let Some(hex) = value.strip_prefix("sha256:") else {
        return false;
    };
    hex.len() == 64 && hex.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn check_residual(
    contest_id: &str,
    field: &str,
    declared: i64,
    computed: i64,
) -> Result<(), RcountCoreError> {
    if declared != computed {
        return Err(RcountCoreError::JurisdictionResidualMismatch {
            contest_id: contest_id.to_string(),
            field: field.to_string(),
            declared,
            computed,
        });
    }
    Ok(())
}

fn check_cvr_field(
    contest_id: &str,
    reporting_unit_id: &str,
    field: &str,
    summary: i64,
    cvr: i64,
) -> Result<(), RcountCoreError> {
    if summary != cvr {
        return Err(RcountCoreError::CvrSummaryMismatch {
            contest_id: contest_id.to_string(),
            reporting_unit_id: reporting_unit_id.to_string(),
            field: field.to_string(),
            summary,
            cvr,
        });
    }
    Ok(())
}

fn is_supported_audit_algorithm_method(method_id: &str) -> bool {
    matches!(
        method_id,
        BRAVO_BALLOT_POLLING_METHOD_ID
            | MINERVA_BALLOT_POLLING_METHOD_ID
            | ATHENA_BALLOT_POLLING_METHOD_ID
            | KAPLAN_MARKOV_COMPARISON_METHOD_ID
            | ALPHA_MARTINGALE_METHOD_ID
            | SHANGRLA_ASSORTER_METHOD_ID
            | STRATIFIED_HYBRID_RLA_METHOD_ID
            | BATCH_COMPARISON_METHOD_ID
            | RAIRE_IRV_METHOD_ID
            | AWAIRE_IRV_METHOD_ID
            | BAYESIAN_TABULATION_AUDIT_METHOD_ID
            | SOBA_OBSERVABLE_BALLOT_AUDIT_METHOD_ID
    )
}

fn is_positive_rational(value: RationalValue) -> bool {
    value.denominator > 0 && value.numerator > 0
}

fn is_non_negative_rational(value: RationalValue) -> bool {
    value.denominator > 0 && value.numerator >= 0
}

fn has_positive_denominator(value: RationalValue) -> bool {
    value.denominator > 0
}

fn rational_gt(lhs: RationalValue, rhs: RationalValue) -> bool {
    (lhs.numerator as i128) * (rhs.denominator as i128)
        > (rhs.numerator as i128) * (lhs.denominator as i128)
}

fn rational_eq(lhs: RationalValue, rhs: RationalValue) -> bool {
    lhs.denominator > 0
        && rhs.denominator > 0
        && (lhs.numerator as i128) * (rhs.denominator as i128)
            == (rhs.numerator as i128) * (lhs.denominator as i128)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn synthetic_shangrla_algorithm_run() -> AuditAlgorithmRun {
        AuditAlgorithmRun {
            run_id: "audit-run:shangrla-toy".to_string(),
            contest_id: "syn-2024-mayor".to_string(),
            method_id: SHANGRLA_ASSORTER_METHOD_ID.to_string(),
            sampling_mode: AuditSamplingMode::WithoutReplacement,
            rcv_elimination_order: Vec::new(),
            risk_limit_ppm: Some(100_000),
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
                kind: AuditAssertionKind::PluralityWinnerLoser,
                assorter_id: "plurality-winner-loser-v1".to_string(),
                assorter_upper_bound: RationalValue {
                    numerator: 1,
                    denominator: 1,
                },
                winner_selection_id: Some("cand-a".to_string()),
                loser_selection_id: Some("cand-b".to_string()),
            }],
            sample_steps: vec![
                AuditSampleStep {
                    step_index: 0,
                    round_index: None,
                    assertion_id: "assertion:cand-a-over-cand-b".to_string(),
                    sample_unit_id: "cvr:P-001:001".to_string(),
                    assorter_value: RationalValue {
                        numerator: 1,
                        denominator: 1,
                    },
                    bet: None,
                    statistic: Some(RationalValue {
                        numerator: 2,
                        denominator: 1,
                    }),
                    p_value_ppm: Some(80_000),
                    ranked_choices: Vec::new(),
                    source_refs: vec!["source:synthetic-audit".to_string()],
                },
                AuditSampleStep {
                    step_index: 1,
                    round_index: None,
                    assertion_id: "assertion:cand-a-over-cand-b".to_string(),
                    sample_unit_id: "cvr:P-001:002".to_string(),
                    assorter_value: RationalValue {
                        numerator: 1,
                        denominator: 2,
                    },
                    bet: None,
                    statistic: Some(RationalValue {
                        numerator: 3,
                        denominator: 1,
                    }),
                    p_value_ppm: Some(50_000),
                    ranked_choices: Vec::new(),
                    source_refs: vec!["source:synthetic-audit".to_string()],
                },
            ],
            decision: AuditAlgorithmDecision::Pass,
            source_refs: vec!["source:synthetic-audit".to_string()],
        }
    }

    #[test]
    fn synthetic_summary_basic_verifies_selection_sums() {
        let package = synthetic_summary_basic_package();
        let report = verify_package(&package).expect("synthetic summary package must verify");
        assert_eq!(report.passed.len(), 3);
        assert!(report.failed.is_empty());
    }

    #[test]
    fn parallel_verifier_matches_serial_reports() {
        let packages = vec![
            synthetic_summary_basic_package(),
            synthetic_canvass_correction_package(),
            synthetic_mail_batch_added_package(),
            synthetic_precinct_split_lineage_package(),
            synthetic_privacy_inclusion_package(),
            synthetic_cvr_summary_package(),
            synthetic_rla_replay_package(),
            synthetic_rla_stopping_package(),
            synthetic_rla_margin_package(),
            synthetic_rla_statistical_package(),
            synthetic_colorado_rla_package(),
            synthetic_california_rla_package(),
            synthetic_manual_audit_package(),
        ];

        for package in packages {
            let serial = verify_package(&package).expect("serial verifier must accept fixture");
            let parallel =
                verify_package_parallel(&package).expect("parallel verifier must accept fixture");
            assert_eq!(parallel, serial);
        }
    }

    #[test]
    fn parallel_verifier_matches_serial_error_for_bad_selection_sum() {
        let package = synthetic_bad_selection_sum_package();
        assert_eq!(
            verify_package_parallel(&package).expect_err("parallel verifier must fail"),
            verify_package(&package).expect_err("serial verifier must fail")
        );
    }

    #[test]
    fn audit_algorithm_transcript_accepts_shangrla_assorter_steps() {
        let mut package = synthetic_summary_basic_package();
        package.audit_algorithm_runs = vec![synthetic_shangrla_algorithm_run()];

        let report = verify_package(&package).expect("audit algorithm transcript must verify");
        assert!(report.passed.iter().any(|pass| {
            pass.equation_id == "audit_algorithm_transcript"
                && pass.reporting_unit_id == "audit-run:shangrla-toy"
        }));
    }

    #[test]
    fn audit_algorithm_transcript_rejects_missing_assertion_step() {
        let mut package = synthetic_summary_basic_package();
        let mut run = synthetic_shangrla_algorithm_run();
        run.sample_steps[0].assertion_id = "assertion:missing".to_string();
        package.audit_algorithm_runs = vec![run];

        assert!(matches!(
            verify_package(&package),
            Err(RcountCoreError::MissingAuditAssertion { .. })
        ));
    }

    #[test]
    fn audit_algorithm_transcript_rejects_out_of_bound_assorter_value() {
        let mut package = synthetic_summary_basic_package();
        let mut run = synthetic_shangrla_algorithm_run();
        run.sample_steps[0].assorter_value = RationalValue {
            numerator: 3,
            denominator: 2,
        };
        package.audit_algorithm_runs = vec![run];

        assert!(matches!(
            verify_package(&package),
            Err(RcountCoreError::InvalidAuditAssorterValue { .. })
        ));
    }

    #[test]
    fn audit_algorithm_transcript_rejects_partial_macro_design() {
        let mut package = synthetic_summary_basic_package();
        let mut run = synthetic_shangrla_algorithm_run();
        run.method_id = KAPLAN_MARKOV_COMPARISON_METHOD_ID.to_string();
        run.macro_ballot_count = Some(100);
        package.audit_algorithm_runs = vec![run];

        assert!(matches!(
            verify_package(&package),
            Err(RcountCoreError::InvalidAuditMacroDesign { .. })
        ));
    }

    #[test]
    fn audit_algorithm_transcript_rejects_invalid_macro_gamma() {
        let mut package = synthetic_summary_basic_package();
        let mut run = synthetic_shangrla_algorithm_run();
        run.method_id = KAPLAN_MARKOV_COMPARISON_METHOD_ID.to_string();
        run.macro_ballot_count = Some(100);
        run.macro_reported_margin = Some(10);
        run.macro_gamma = Some(RationalValue {
            numerator: 1,
            denominator: 1,
        });
        package.audit_algorithm_runs = vec![run];

        assert!(matches!(
            verify_package(&package),
            Err(RcountCoreError::InvalidAuditMacroDesign { .. })
        ));
    }

    #[test]
    fn kaplan_markov_macro_package_verifies_algorithm_transcript() {
        let package = synthetic_kaplan_markov_macro_package();
        let report = verify_package(&package).expect("MACRO package must verify");
        assert!(report.passed.iter().any(|pass| {
            pass.equation_id == "audit_algorithm_transcript"
                && pass.reporting_unit_id == "audit-run:kaplan-markov-macro-pass"
        }));
    }

    #[test]
    fn minerva_round_one_package_verifies_algorithm_transcript() {
        let package = synthetic_minerva_round_one_package();
        let report = verify_package(&package).expect("Minerva package must verify");
        assert!(report.passed.iter().any(|pass| {
            pass.equation_id == "audit_algorithm_transcript"
                && pass.reporting_unit_id == "audit-run:minerva-round-one-pass"
        }));
    }

    #[test]
    fn minerva_multi_round_package_verifies_algorithm_transcript() {
        let package = synthetic_minerva_multi_round_package();
        let report = verify_package(&package).expect("multi-round Minerva package must verify");
        assert!(report.passed.iter().any(|pass| {
            pass.equation_id == "audit_algorithm_transcript"
                && pass.reporting_unit_id == "audit-run:minerva-multi-round-pass"
        }));
        assert_eq!(
            package.audit_algorithm_runs[0].sample_steps[4].round_index,
            Some(0)
        );
        assert_eq!(
            package.audit_algorithm_runs[0].sample_steps[5].round_index,
            Some(1)
        );
    }

    #[test]
    fn athena_boundary_package_verifies_algorithm_transcript() {
        let package = synthetic_athena_boundary_package();
        let report = verify_package(&package).expect("Athena boundary package must verify");
        assert!(report.passed.iter().any(|pass| {
            pass.equation_id == "audit_algorithm_transcript"
                && pass.reporting_unit_id == "audit-run:athena-boundary"
        }));
        assert_eq!(
            package.audit_algorithm_runs[0].method_id,
            ATHENA_BALLOT_POLLING_METHOD_ID
        );
    }

    #[test]
    fn stratified_hybrid_package_verifies_component_references() {
        let package = synthetic_stratified_hybrid_package();
        let report = verify_package(&package).expect("stratified package must verify");
        assert!(report.passed.iter().any(|pass| {
            pass.equation_id == "audit_algorithm_transcript"
                && pass.reporting_unit_id == "audit-run:stratified-hybrid-boundary"
        }));
        let run = package
            .audit_algorithm_runs
            .iter()
            .find(|run| run.method_id == STRATIFIED_HYBRID_RLA_METHOD_ID)
            .expect("stratified run must be present");
        assert_eq!(run.strata.len(), 2);
        assert_eq!(
            run.combining_rule_id.as_deref(),
            Some("suite-nuisance-boundary-v1")
        );
        assert_eq!(
            run.nuisance_parameter,
            Some(RationalValue {
                numerator: 1,
                denominator: 2,
            })
        );
        assert_eq!(run.strata[0].allocation_ppm, Some(500_000));
        assert_eq!(run.strata[1].allocation_ppm, Some(500_000));
    }

    #[test]
    fn stratified_hybrid_package_rejects_missing_component() {
        let package = synthetic_bad_stratified_hybrid_package();
        assert!(matches!(
            verify_package(&package),
            Err(RcountCoreError::MissingStratifiedHybridComponent { .. })
        ));
    }

    #[test]
    fn stratified_hybrid_package_rejects_flattened_stratum() {
        let package = synthetic_flattened_stratified_hybrid_package();
        assert!(matches!(
            verify_package(&package),
            Err(RcountCoreError::InvalidStratifiedHybridDesign { .. })
        ));
    }

    #[test]
    fn raire_boundary_package_verifies_ranked_choice_surface() {
        let package = synthetic_raire_boundary_package();
        let report = verify_package(&package).expect("RAIRE boundary package must verify");
        assert!(report.passed.iter().any(|pass| {
            pass.equation_id == "audit_algorithm_transcript"
                && pass.reporting_unit_id == "audit-run:raire-irv-boundary"
        }));
        assert_eq!(
            package.audit_algorithm_runs[0].rcv_elimination_order,
            vec![
                "cand-c".to_string(),
                "cand-b".to_string(),
                "cand-a".to_string(),
            ]
        );
    }

    #[test]
    fn awaire_boundary_package_verifies_ranked_choice_surface() {
        let package = synthetic_awaire_boundary_package();
        let report = verify_package(&package).expect("AWAIRE boundary package must verify");
        assert!(report.passed.iter().any(|pass| {
            pass.equation_id == "audit_algorithm_transcript"
                && pass.reporting_unit_id == "audit-run:awaire-irv-boundary"
        }));
        assert_eq!(
            package.audit_algorithm_runs[0].method_id,
            AWAIRE_IRV_METHOD_ID
        );
    }

    #[test]
    fn ranked_choice_boundary_package_rejects_duplicate_ranked_choice() {
        let package = synthetic_bad_raire_boundary_package();
        assert!(matches!(
            verify_package(&package),
            Err(RcountCoreError::InvalidRankedChoiceSample { .. })
        ));
    }

    #[test]
    fn bayesian_tabulation_boundary_package_verifies_analytic_surface() {
        let package = synthetic_bayesian_tabulation_boundary_package();
        let report =
            verify_package(&package).expect("Bayesian tabulation boundary package must verify");
        assert!(report.passed.iter().any(|pass| {
            pass.equation_id == "audit_algorithm_transcript"
                && pass.reporting_unit_id == "audit-run:bayesian-tabulation-boundary"
        }));
        assert_eq!(
            package.audit_algorithm_runs[0].posterior_winner_probability_ppm,
            Some(958_000)
        );
        assert_eq!(
            package.audit_algorithm_runs[0].posterior_risk_ppm,
            Some(42_000)
        );
    }

    #[test]
    fn bayesian_tabulation_boundary_package_rejects_invalid_posterior_risk() {
        let package = synthetic_bad_bayesian_tabulation_boundary_package();
        assert!(matches!(
            verify_package(&package),
            Err(RcountCoreError::InvalidBayesianAuditDesign { .. })
        ));
    }

    #[test]
    fn soba_observable_ballot_boundary_package_verifies_opening_linkage() {
        let package = synthetic_soba_observable_ballot_boundary_package();
        let report = verify_package(&package).expect("SOBA boundary package must verify");
        assert!(report.passed.iter().any(|pass| {
            pass.equation_id == "audit_algorithm_transcript"
                && pass.reporting_unit_id == "audit-run:soba-observable-ballot-boundary"
        }));
        assert!(package.inclusion_proofs[0].candidate_selections.is_empty());
        assert_eq!(
            package.audit_algorithm_runs[0].assertions[0].kind,
            AuditAssertionKind::ObservableBallotLinkage
        );
    }

    #[test]
    fn soba_observable_ballot_boundary_package_rejects_missing_opening() {
        let package = synthetic_missing_soba_opening_package();
        assert!(matches!(
            verify_package(&package),
            Err(RcountCoreError::MissingObservableBallotOpening { .. })
        ));
    }

    #[test]
    fn synthetic_summary_basic_verifies_jurisdiction_total() {
        let package = synthetic_summary_basic_package();
        let passes =
            verify_jurisdiction_total("syn-2024-mayor", "syn:jurisdiction:SYN", &package.summaries)
                .expect("jurisdiction total must verify");
        assert_eq!(passes[0].equation_id, "jurisdiction_contest_total");
    }

    #[test]
    fn synthetic_canvass_correction_verifies_both_status_snapshots() {
        let package = synthetic_canvass_correction_package();
        let report = verify_package(&package).expect("canvass correction package must verify");
        assert_eq!(
            report
                .passed
                .iter()
                .filter(|pass| pass.equation_id == "contest_selection_sum")
                .count(),
            6
        );
        let jurisdiction_passes =
            verify_jurisdiction_total("syn-2024-mayor", "syn:jurisdiction:SYN", &package.summaries)
                .expect("both status snapshots must reconcile");
        assert_eq!(jurisdiction_passes.len(), 2);
    }

    #[test]
    fn synthetic_mail_batch_added_verifies_batch_summaries() {
        let package = synthetic_mail_batch_added_package();
        let report = verify_package(&package).expect("mail batch package must verify");
        assert_eq!(
            report
                .passed
                .iter()
                .filter(|pass| pass.equation_id == "batch_summary_total")
                .count(),
            3
        );
        assert_eq!(
            report
                .passed
                .iter()
                .filter(|pass| pass.equation_id == "accepted_ballots")
                .count(),
            3
        );
        let jurisdiction_passes =
            verify_jurisdiction_total("syn-2024-mayor", "syn:jurisdiction:SYN", &package.summaries)
                .expect("batched summaries must roll up");
        assert_eq!(jurisdiction_passes.len(), 1);
    }

    #[test]
    fn batch_summary_total_catches_missing_batch() {
        let package = synthetic_missing_batch_package();
        let err = verify_batch_summary_totals(&package).expect_err("missing batch must fail");
        assert!(matches!(err, RcountCoreError::MissingBatch { .. }));
    }

    #[test]
    fn synthetic_precinct_split_lineage_verifies_split_and_merge() {
        let package = synthetic_precinct_split_lineage_package();
        let report = verify_package(&package).expect("lineage package must verify");
        assert_eq!(
            report
                .passed
                .iter()
                .filter(|pass| pass.equation_id == "lineage_conservation")
                .count(),
            2
        );
    }

    #[test]
    fn rhist_reference_declared_verifies() {
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

        let report = verify_package(&package).expect("RHIST reference must verify");
        assert!(report.passed.iter().any(|pass| {
            pass.equation_id == "rhist_reference_declared"
                && pass.reporting_unit_id == "rhist:real-ri-tract-unchanged"
        }));
    }

    #[test]
    fn rhist_reference_consumes_split_merge_fixture_package_hash() {
        let mut package = synthetic_summary_basic_package();
        let package_hash = rhist_fixture_package_hash("l2-three-cycle");
        package.rhist_refs = vec![RhistReference {
            reference_id: "rhist:syn-l2-three-cycle".to_string(),
            package_hash: package_hash.clone(),
            package_path: Some("docs/fixtures/rhist/l2-three-cycle".to_string()),
            cycle_ids: vec![
                "syn-2024-general".to_string(),
                "syn-2026-general".to_string(),
                "syn-2028-general".to_string(),
            ],
            role: "unit-lineage".to_string(),
            note: Some("References RHIST rename/split/merge fixture by package hash.".to_string()),
        }];

        let report = verify_package(&package).expect("RHIST fixture reference must verify");
        assert!(report.passed.iter().any(|pass| {
            pass.equation_id == "rhist_reference_declared"
                && pass.reporting_unit_id == "rhist:syn-l2-three-cycle"
        }));
        assert_eq!(
            package.rhist_refs[0].package_hash,
            SYN_RHIST_L2_PACKAGE_HASH
        );
    }

    #[test]
    fn synthetic_summary_basic_base_references_verify_together() {
        let package = synthetic_summary_basic_package_with_base_references();
        let report = verify_package(&package).expect("base references must verify");

        assert_eq!(package.rctx_refs[0].context_hash, SYN_RCTX_L0_CONTEXT_HASH);
        assert_eq!(
            package.rctx_refs[0].crosswalk_hash.as_deref(),
            Some(SYN_RCTX_L0_CROSSWALK_HASH)
        );
        assert_eq!(
            package.rhist_refs[0].package_hash,
            SYN_RHIST_L2_PACKAGE_HASH
        );
        assert!(report
            .passed
            .iter()
            .any(|pass| pass.equation_id == "rctx_reference_declared"));
        assert!(report
            .passed
            .iter()
            .any(|pass| pass.equation_id == "rhist_reference_declared"));
    }

    #[test]
    fn rhist_reference_rejects_bad_hash() {
        let mut package = synthetic_summary_basic_package();
        package.rhist_refs = vec![RhistReference {
            reference_id: "rhist:bad-hash".to_string(),
            package_hash: "not-a-hash".to_string(),
            package_path: None,
            cycle_ids: vec!["cycle:one".to_string()],
            role: "unit-lineage".to_string(),
            note: None,
        }];

        assert!(matches!(
            verify_package(&package),
            Err(RcountCoreError::InvalidRhistPackageHash { .. })
        ));
    }

    #[test]
    fn rhist_reference_requires_cycles_and_supported_role() {
        let mut package = synthetic_summary_basic_package();
        package.rhist_refs = vec![RhistReference {
            reference_id: "rhist:no-cycles".to_string(),
            package_hash: "sha256:ccbddf423aa4ac08b0d45c4ac0b9db411293ea41fef3ac8fa93f9de9e85f66bb"
                .to_string(),
            package_path: None,
            cycle_ids: vec![],
            role: "unit-lineage".to_string(),
            note: None,
        }];
        assert!(matches!(
            verify_package(&package),
            Err(RcountCoreError::EmptyRhistCycleRefs { .. })
        ));

        package.rhist_refs[0].cycle_ids = vec!["cycle:one".to_string()];
        package.rhist_refs[0].role = "freeform-history".to_string();
        assert!(matches!(
            verify_package(&package),
            Err(RcountCoreError::UnsupportedRhistReferenceRole { .. })
        ));
    }

    fn rhist_fixture_package_hash(name: &str) -> String {
        let manifest_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("docs")
            .join("fixtures")
            .join("rhist")
            .join(name)
            .join("manifest.json");
        let manifest: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(manifest_path).unwrap()).unwrap();
        manifest["package_content_hash"]
            .as_str()
            .unwrap()
            .to_string()
    }

    #[test]
    fn rctx_reference_declared_verifies() {
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

        let report = verify_package(&package).expect("RCTX reference must verify");
        assert!(report.passed.iter().any(|pass| {
            pass.equation_id == "rctx_reference_declared"
                && pass.reporting_unit_id == "rctx:summary-basic-context"
        }));
    }

    #[test]
    fn rctx_reference_rejects_bad_hashes_and_role() {
        let mut package = synthetic_summary_basic_package();
        package.rctx_refs = vec![RctxReference {
            reference_id: "rctx:bad-context".to_string(),
            context_hash: "not-a-hash".to_string(),
            context_path: None,
            crosswalk_hash: None,
            crosswalk_path: None,
            role: "unit-context".to_string(),
            note: None,
        }];
        assert!(matches!(
            verify_package(&package),
            Err(RcountCoreError::InvalidRctxContextHash { .. })
        ));

        package.rctx_refs[0].context_hash =
            "sha256:1111111111111111111111111111111111111111111111111111111111111111".to_string();
        package.rctx_refs[0].crosswalk_hash = Some("not-a-hash".to_string());
        assert!(matches!(
            verify_package(&package),
            Err(RcountCoreError::InvalidRctxCrosswalkHash { .. })
        ));

        package.rctx_refs[0].crosswalk_hash = None;
        package.rctx_refs[0].role = "map-render".to_string();
        assert!(matches!(
            verify_package(&package),
            Err(RcountCoreError::UnsupportedRctxReferenceRole { .. })
        ));
    }

    #[test]
    fn lineage_conservation_catches_missing_current_unit() {
        let package = synthetic_bad_lineage_package();
        let err = verify_lineage_conservation(&package).expect_err("bad lineage must fail");
        assert!(matches!(
            err,
            RcountCoreError::MissingCurrentLineageUnit { .. }
        ));
    }

    #[test]
    fn synthetic_privacy_inclusion_proof_verifies() {
        let package = synthetic_privacy_inclusion_package();
        let report = verify_package(&package).expect("privacy inclusion proof must verify");
        assert_eq!(
            report
                .passed
                .iter()
                .filter(|pass| pass.equation_id == "proof_privacy_gate")
                .count(),
            1
        );
    }

    #[test]
    fn choice_bearing_proof_fails_privacy_gate() {
        let package = synthetic_choice_bearing_proof_package();
        let err = verify_proof_privacy(&package).expect_err("choice-bearing proof must fail");
        assert!(matches!(err, RcountCoreError::ChoiceBearingProof { .. }));
    }

    #[test]
    fn synthetic_cvr_summary_verifies_against_summaries() {
        let package = synthetic_cvr_summary_package();
        let report = verify_package(&package).expect("CVR summary package must verify");
        assert_eq!(
            report
                .passed
                .iter()
                .filter(|pass| pass.equation_id == "cvr_summary_total")
                .count(),
            2
        );
    }

    #[test]
    fn bad_cvr_summary_fails_cvr_reconciliation() {
        let package = synthetic_bad_cvr_summary_package();
        let err = verify_cvr_summary_reconciliation(&package)
            .expect_err("bad CVR summary package must fail");
        assert!(matches!(err, RcountCoreError::CvrSummaryMismatch { .. }));
    }

    #[test]
    fn rla_replay_package_verifies_sample() {
        let package = synthetic_rla_replay_package();
        let report = verify_package(&package).expect("RLA replay package must verify");
        assert!(report
            .passed
            .iter()
            .any(|pass| pass.equation_id == "rla_sampler_replay"));
        assert_eq!(package.rla_audits[0].sample_draws.len(), 12);
    }

    #[test]
    fn rla_replay_fails_on_tampered_sample_draw() {
        let package = synthetic_bad_rla_replay_package();
        let err = verify_rla_sampler_replay(&package)
            .expect_err("bad RLA replay package must fail sample replay");
        assert!(matches!(err, RcountCoreError::RlaSampleMismatch { .. }));
    }

    #[test]
    fn rla_stopping_package_verifies_observations() {
        let package = synthetic_rla_stopping_package();
        let report = verify_package(&package).expect("RLA stopping package must verify");
        assert!(report
            .passed
            .iter()
            .any(|pass| pass.equation_id == "rla_stopping_rule"));
    }

    #[test]
    fn rla_stopping_fails_when_declared_pass_has_discrepancy() {
        let package = synthetic_bad_rla_stopping_package();
        let err = verify_rla_stopping_rules(&package)
            .expect_err("bad RLA stopping package must fail stopping rule");
        assert!(matches!(
            err,
            RcountCoreError::RlaStoppingStatusMismatch { .. }
        ));
    }

    #[test]
    fn rla_discrepancy_package_verifies_declared_taxonomy() {
        let package = synthetic_rla_discrepancy_package();
        let report = verify_package(&package).expect("RLA discrepancy package must verify");
        assert!(report
            .passed
            .iter()
            .any(|pass| pass.equation_id == "rla_stopping_rule"));
    }

    #[test]
    fn rla_discrepancy_fails_when_declared_kind_is_wrong() {
        let package = synthetic_bad_rla_discrepancy_package();
        let err = verify_rla_stopping_rules(&package)
            .expect_err("bad RLA discrepancy package must fail taxonomy check");
        assert!(matches!(
            err,
            RcountCoreError::RlaDiscrepancyMismatch { .. }
        ));
    }

    #[test]
    fn rla_margin_package_verifies_reported_margin_metadata() {
        let package = synthetic_rla_margin_package();
        let report = verify_package(&package).expect("RLA margin package must verify");
        assert!(report
            .passed
            .iter()
            .any(|pass| pass.equation_id == "rla_margin_metadata"));
    }

    #[test]
    fn rla_margin_fails_when_declared_margin_drifts() {
        let package = synthetic_bad_rla_margin_package();
        let err = verify_rla_margin_metadata(&package)
            .expect_err("bad RLA margin package must fail margin metadata");
        assert!(matches!(
            err,
            RcountCoreError::RlaReportedMarginMismatch { .. }
        ));
    }

    #[test]
    fn rla_statistical_package_verifies_risk_estimate() {
        let package = synthetic_rla_statistical_package();
        let report = verify_package(&package).expect("RLA statistical package must verify");
        assert!(report
            .passed
            .iter()
            .any(|pass| pass.equation_id == "rla_stopping_rule"));
        assert_eq!(package.rla_audits[0].declared_risk_ppm, Some(1303));
    }

    #[test]
    fn rla_statistical_fails_when_declared_risk_drifts() {
        let package = synthetic_bad_rla_statistical_package();
        let err = verify_rla_stopping_rules(&package)
            .expect_err("bad RLA statistical package must fail risk estimate");
        assert!(matches!(
            err,
            RcountCoreError::RlaRiskEstimateMismatch { .. }
        ));
    }

    #[test]
    fn colorado_rla_package_verifies_jurisdiction_adapter() {
        let package = synthetic_colorado_rla_package();
        let report = verify_package(&package).expect("Colorado-style RLA package must verify");
        assert!(report
            .passed
            .iter()
            .any(|pass| pass.equation_id == "rla_jurisdiction_adapter"));
        assert_eq!(
            package.rla_audits[0].jurisdiction_method_id.as_deref(),
            Some(COLORADO_RLA_METHOD_ID)
        );
    }

    #[test]
    fn colorado_rla_fails_when_seed_is_not_twenty_digits() {
        let package = synthetic_bad_colorado_rla_package();
        let err = verify_rla_jurisdiction_adapters(&package)
            .expect_err("bad Colorado-style RLA package must fail jurisdiction adapter");
        assert!(matches!(
            err,
            RcountCoreError::InvalidColoradoRlaSeed { .. }
        ));
    }

    #[test]
    fn california_rla_package_verifies_public_tool_adapter() {
        let package = synthetic_california_rla_package();
        let report = verify_package(&package).expect("California-style RLA package must verify");
        assert!(report
            .passed
            .iter()
            .any(|pass| pass.equation_id == "rla_jurisdiction_adapter"));
        assert_eq!(
            package.rla_audits[0].ballot_manifest_format_id.as_deref(),
            Some(CALIFORNIA_BALLOT_MANIFEST_FORMAT_ID)
        );
    }

    #[test]
    fn california_rla_fails_when_source_url_is_not_public_url() {
        let package = synthetic_bad_california_rla_package();
        let err = verify_rla_jurisdiction_adapters(&package)
            .expect_err("bad California-style RLA package must fail jurisdiction adapter");
        assert!(matches!(
            err,
            RcountCoreError::InvalidRlaSoftwareSourceUrl { .. }
        ));
    }

    #[test]
    fn manual_audit_package_verifies_hand_count_totals() {
        let package = synthetic_manual_audit_package();
        let report = verify_package(&package).expect("manual audit package must verify");
        assert!(report
            .passed
            .iter()
            .any(|pass| pass.equation_id == "manual_audit_reconciliation"));
    }

    #[test]
    fn manual_audit_fails_when_hand_count_exceeds_tolerance() {
        let package = synthetic_bad_manual_audit_package();
        let err = verify_manual_audits(&package)
            .expect_err("bad manual audit package must fail reconciliation");
        assert!(matches!(
            err,
            RcountCoreError::ManualAuditStatusMismatch { .. }
        ));
    }

    #[test]
    fn batch_comparison_package_verifies_overstatement() {
        let package = synthetic_batch_comparison_package();
        let report = verify_package(&package).expect("batch comparison package must verify");
        assert!(report
            .passed
            .iter()
            .any(|pass| pass.equation_id == "batch_comparison_overstatement"
                && pass.reporting_unit_id == "batch:P-001:election-day"));
    }

    #[test]
    fn batch_comparison_algorithm_links_to_verified_overstatement() {
        let package = synthetic_batch_comparison_algorithm_package();
        let report =
            verify_package(&package).expect("batch comparison algorithm package must verify");
        assert!(report.passed.iter().any(|pass| {
            pass.equation_id == "batch_comparison_algorithm_linkage"
                && pass.reporting_unit_id == "audit-run:batch-comparison-taint-linkage"
        }));
    }

    #[test]
    fn derives_batch_comparison_algorithm_run_from_sample_order() {
        let package = synthetic_batch_comparison_package();
        let run = derive_batch_comparison_algorithm_run(
            &package,
            "audit-run:batch-comparison-derived",
            "syn-2024-mayor",
            300_000,
            &["batch:P-001:election-day".to_string()],
            AuditAlgorithmDecision::Continue,
        )
        .expect("batch comparison run must derive from package audits");

        assert_eq!(run.method_id, BATCH_COMPARISON_METHOD_ID);
        assert_eq!(
            run.sample_steps[0].sample_unit_id,
            "batch:P-001:election-day"
        );
        assert_eq!(
            run.sample_steps[0].assorter_value,
            RationalValue {
                numerator: 2,
                denominator: 5
            }
        );
        assert!(run.sample_steps[0]
            .source_refs
            .contains(&"batch-comparison:P-001-election-day".to_string()));
        assert!(run
            .source_refs
            .contains(&"source:synthetic-batch-hand-tally".to_string()));
    }

    #[test]
    fn batch_comparison_algorithm_derivation_rejects_missing_sampled_batch() {
        let package = synthetic_batch_comparison_package();
        let err = derive_batch_comparison_algorithm_run(
            &package,
            "audit-run:batch-comparison-derived",
            "syn-2024-mayor",
            300_000,
            &["batch:P-404".to_string()],
            AuditAlgorithmDecision::Continue,
        )
        .expect_err("missing sampled batch must fail derivation");

        assert!(matches!(
            err,
            RcountCoreError::MissingBatchComparisonAlgorithmEvidence { .. }
        ));
    }

    #[test]
    fn batch_comparison_algorithm_fails_when_taint_drifts() {
        let package = synthetic_bad_batch_comparison_algorithm_package();
        let err = verify_audit_algorithm_runs(&package)
            .expect_err("bad batch comparison algorithm taint must fail linkage");
        assert!(matches!(
            err,
            RcountCoreError::BatchComparisonAlgorithmTaintMismatch { .. }
        ));
    }

    #[test]
    fn batch_comparison_fails_when_overstatement_drifts() {
        let package = synthetic_bad_batch_comparison_package();
        let err = verify_batch_comparison_audits(&package)
            .expect_err("bad batch comparison package must fail overstatement check");
        assert!(matches!(
            err,
            RcountCoreError::BatchComparisonOverstatementMismatch { .. }
        ));
    }

    #[test]
    fn batch_comparison_fails_when_hand_tally_is_missing() {
        let package = synthetic_missing_hand_tally_batch_comparison_package();
        let err = verify_batch_comparison_audits(&package)
            .expect_err("missing hand tally must fail batch comparison check");
        assert!(matches!(
            err,
            RcountCoreError::MissingBatchComparisonHandTally { .. }
        ));
    }

    #[test]
    fn batch_comparison_fails_when_batch_size_drifts() {
        let package = synthetic_batch_size_drift_comparison_package();
        let err = verify_batch_comparison_audits(&package)
            .expect_err("batch size drift must fail before overstatement check");
        assert!(matches!(
            err,
            RcountCoreError::BatchComparisonBatchSizeMismatch { .. }
        ));
    }

    #[test]
    fn canvass_correction_requires_public_event_and_snapshots() {
        let mut package = synthetic_canvass_correction_package();
        let pass = verify_canvass_correction_event(&package).unwrap();
        assert_eq!(pass.equation_id, "canvass_correction_event");

        package.status_events.clear();
        let err = verify_canvass_correction_event(&package)
            .expect_err("missing correction event must fail");
        assert!(matches!(
            err,
            RcountCoreError::MissingCanvassCorrectionEvent
        ));
    }

    #[test]
    fn bad_arithmetic_fails_with_specific_equation_error() {
        let package = synthetic_bad_selection_sum_package();
        let err = verify_package(&package).expect_err("bad counted ballot total must fail");
        assert!(matches!(
            err,
            RcountCoreError::ContestSelectionSumMismatch { .. }
        ));
    }

    #[test]
    fn tampered_jurisdiction_total_fails() {
        let mut package = synthetic_summary_basic_package();
        let total = package
            .summaries
            .iter_mut()
            .find(|summary| summary.reporting_unit_id == "syn:jurisdiction:SYN")
            .unwrap();
        total.totals[0].votes += 1;
        let err =
            verify_jurisdiction_total("syn-2024-mayor", "syn:jurisdiction:SYN", &package.summaries)
                .expect_err("tampered jurisdiction total must fail");
        assert!(matches!(
            err,
            RcountCoreError::JurisdictionSelectionMismatch { .. }
        ));
    }

    #[test]
    fn record_hash_is_stable_for_equivalent_json_key_order() {
        let a = serde_json::json!({"b": 2, "a": {"d": 4, "c": 3}});
        let b = serde_json::json!({"a": {"c": 3, "d": 4}, "b": 2});
        assert_eq!(
            canonical_hash(RECORD_HASH_PREFIX, &a).unwrap(),
            canonical_hash(RECORD_HASH_PREFIX, &b).unwrap()
        );
    }

    #[test]
    fn package_content_hash_has_rcount_prefix() {
        let package = synthetic_summary_basic_package();
        let hash = package_content_hash(&package).unwrap();
        assert!(hash.starts_with("sha256:"));
        assert_eq!(hash.len(), "sha256:".len() + 64);
    }
}
