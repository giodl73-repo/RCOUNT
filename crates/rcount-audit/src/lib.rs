use rcount_core::{
    package_content_hash, verify_canvass_correction_event, verify_jurisdiction_total,
    verify_package, AuditAlgorithmDecision, AuditAlgorithmRun, EquationPass, RationalValue,
    RcountCoreError, RcountPackage, ReportingUnitKind, StatusEventType, ALPHA_MARTINGALE_METHOD_ID,
    ATHENA_BALLOT_POLLING_METHOD_ID, AWAIRE_IRV_METHOD_ID, BATCH_COMPARISON_METHOD_ID,
    BAYESIAN_TABULATION_AUDIT_METHOD_ID, BRAVO_BALLOT_POLLING_METHOD_ID,
    KAPLAN_MARKOV_COMPARISON_METHOD_ID, MINERVA_BALLOT_POLLING_METHOD_ID, RAIRE_IRV_METHOD_ID,
    SOBA_OBSERVABLE_BALLOT_AUDIT_METHOD_ID, STRATIFIED_HYBRID_RLA_METHOD_ID,
};
use rcount_io::{read_package_dir, verify_source_index, RcountIoError, RcountManifest};
use rcount_stats::{
    replay_bravo_ballot_polling, replay_fixed_bet_bounded_mean_martingale,
    replay_kaplan_markov_macro_bound, replay_kaplan_markov_taint_product,
    replay_minerva_ballot_polling_rounds, BoundedMeanMartingaleConfig,
    BoundedMeanMartingaleObservation, BravoObservation, KaplanMarkovMacroConfig,
    MinervaRoundObservationSet, Rational, RcountStatsError,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use thiserror::Error;

pub const RCOUNT_AUDIT_TRANSCRIPT_VERSION: &str = "rcount-audit-transcript-v1";

#[derive(Debug, Error)]
pub enum RcountAuditError {
    #[error("io error: {0}")]
    Io(#[from] RcountIoError),
    #[error("core error: {0}")]
    Core(#[from] rcount_core::RcountCoreError),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("filesystem error: {0}")]
    Fs(#[from] std::io::Error),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum VerificationStatus {
    Pass,
    Fail,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckResult {
    pub equation_id: String,
    pub status: VerificationStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contest_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reporting_unit_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerificationTranscript {
    pub transcript_version: String,
    pub verifier: String,
    pub status: VerificationStatus,
    pub package_content_hash: String,
    pub manifest_content_hash: String,
    pub checks: Vec<CheckResult>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AlgorithmReplayStatus {
    Pass,
    Fail,
    Boundary,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AlgorithmReplayStep {
    pub step_index: u32,
    pub assertion_id: String,
    pub sample_unit_id: String,
    pub statistic: RationalValue,
    pub p_value_ppm: u32,
    pub stop: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AlgorithmReplayTranscript {
    pub run_id: String,
    pub method_id: String,
    pub status: AlgorithmReplayStatus,
    pub decision: AuditAlgorithmDecision,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub computed_decision: Option<AuditAlgorithmDecision>,
    pub steps: Vec<AlgorithmReplayStep>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub boundary: Option<String>,
}

pub fn replay_audit_algorithm_statistics(run: &AuditAlgorithmRun) -> AlgorithmReplayTranscript {
    match run.method_id.as_str() {
        BRAVO_BALLOT_POLLING_METHOD_ID => replay_bravo_run(run),
        ALPHA_MARTINGALE_METHOD_ID => replay_alpha_run(run),
        MINERVA_BALLOT_POLLING_METHOD_ID => replay_minerva_run(run),
        ATHENA_BALLOT_POLLING_METHOD_ID => boundary_algorithm_transcript(
            run,
            "Athena round risk calculation is recorded but not replayed",
        ),
        KAPLAN_MARKOV_COMPARISON_METHOD_ID => replay_kaplan_markov_run(run),
        BATCH_COMPARISON_METHOD_ID => replay_comparison_taint_product_run(
            run,
            "batch-comparison taint-product replay requires risk_limit_ppm",
        ),
        STRATIFIED_HYBRID_RLA_METHOD_ID => boundary_algorithm_transcript(
            run,
            "stratified/hybrid combined-risk replay is recorded but not replayed",
        ),
        RAIRE_IRV_METHOD_ID => boundary_algorithm_transcript(
            run,
            "RAIRE IRV assertion replay is recorded but not replayed",
        ),
        AWAIRE_IRV_METHOD_ID => boundary_algorithm_transcript(
            run,
            "AWAIRE IRV adaptive replay is recorded but not replayed",
        ),
        BAYESIAN_TABULATION_AUDIT_METHOD_ID => boundary_algorithm_transcript(
            run,
            "Bayesian tabulation posterior analytics are recorded but not risk-limiting replay",
        ),
        SOBA_OBSERVABLE_BALLOT_AUDIT_METHOD_ID => boundary_algorithm_transcript(
            run,
            "SOBA observable-ballot linkage is recorded but not comparison-risk replay",
        ),
        _ => boundary_algorithm_transcript(run, "method does not yet have statistical replay"),
    }
}

pub fn verify_package_dir(dir: &Path) -> VerificationTranscript {
    match read_package_dir(dir) {
        Ok((manifest, package)) => verify_loaded_package(dir, &manifest, &package),
        Err(err) => VerificationTranscript {
            transcript_version: RCOUNT_AUDIT_TRANSCRIPT_VERSION.to_string(),
            verifier: "rcount-audit".to_string(),
            status: VerificationStatus::Fail,
            package_content_hash: "<unavailable>".to_string(),
            manifest_content_hash: "<unavailable>".to_string(),
            checks: vec![CheckResult {
                equation_id: "package_read".to_string(),
                status: VerificationStatus::Fail,
                contest_id: None,
                reporting_unit_id: None,
                error: Some(err.to_string()),
            }],
        },
    }
}

pub fn write_verification_transcript(
    dir: &Path,
    transcript: &VerificationTranscript,
) -> Result<(), RcountAuditError> {
    let transcript_dir = dir.join("transcripts");
    fs::create_dir_all(&transcript_dir)?;
    let bytes = serde_json::to_vec_pretty(transcript)?;
    fs::write(transcript_dir.join("verify-transcript.json"), bytes)?;
    Ok(())
}

fn replay_bravo_run(run: &AuditAlgorithmRun) -> AlgorithmReplayTranscript {
    let Some(risk_limit_ppm) = run.risk_limit_ppm else {
        return boundary_algorithm_transcript(run, "BRAVO replay requires risk_limit_ppm");
    };
    let Some(winner_votes) = run.reported_winner_votes else {
        return boundary_algorithm_transcript(run, "BRAVO replay requires reported_winner_votes");
    };
    let Some(loser_votes) = run.reported_loser_votes else {
        return boundary_algorithm_transcript(run, "BRAVO replay requires reported_loser_votes");
    };

    let observations = run
        .sample_steps
        .iter()
        .map(bravo_observation_from_step)
        .collect::<Result<Vec<_>, _>>();
    let observations = match observations {
        Ok(observations) => observations,
        Err(err) => return boundary_algorithm_transcript(run, err),
    };

    match replay_bravo_ballot_polling(winner_votes, loser_votes, risk_limit_ppm, &observations) {
        Ok(replay) => build_replay_transcript(
            run,
            replay
                .steps
                .into_iter()
                .zip(run.sample_steps.iter())
                .map(|(computed, declared)| ComputedStep {
                    declared,
                    statistic: computed.likelihood_ratio,
                    p_value_ppm: computed.p_value_ppm,
                    stop: computed.stop,
                })
                .collect(),
            if replay.stopped {
                AuditAlgorithmDecision::Pass
            } else {
                AuditAlgorithmDecision::Continue
            },
        ),
        Err(err) => boundary_algorithm_transcript(run, &err.to_string()),
    }
}

fn replay_alpha_run(run: &AuditAlgorithmRun) -> AlgorithmReplayTranscript {
    let Some(risk_limit_ppm) = run.risk_limit_ppm else {
        return boundary_algorithm_transcript(run, "ALPHA replay requires risk_limit_ppm");
    };
    if run.assertions.len() != 1 {
        return boundary_algorithm_transcript(run, "ALPHA replay currently requires one assertion");
    }
    let assertion = &run.assertions[0];
    let upper_bound = match rational_from_core(assertion.assorter_upper_bound) {
        Ok(value) => value,
        Err(err) => return boundary_algorithm_transcript(run, &err.to_string()),
    };
    let null_mean = match upper_bound.checked_div(Rational::new(2, 1).unwrap()) {
        Ok(value) => value,
        Err(err) => return boundary_algorithm_transcript(run, &err.to_string()),
    };

    let observations = run
        .sample_steps
        .iter()
        .map(|step| {
            let Some(bet) = step.bet else {
                return Err("ALPHA replay requires bet on every sample step".to_string());
            };
            Ok(BoundedMeanMartingaleObservation {
                value: rational_from_core(step.assorter_value).map_err(|err| err.to_string())?,
                bet: rational_from_core(bet).map_err(|err| err.to_string())?,
            })
        })
        .collect::<Result<Vec<_>, _>>();
    let observations = match observations {
        Ok(observations) => observations,
        Err(err) => return boundary_algorithm_transcript(run, &err),
    };

    match replay_fixed_bet_bounded_mean_martingale(
        BoundedMeanMartingaleConfig {
            null_mean,
            upper_bound,
            risk_limit_ppm,
        },
        &observations,
    ) {
        Ok(replay) => build_replay_transcript(
            run,
            replay
                .steps
                .into_iter()
                .zip(run.sample_steps.iter())
                .map(|(computed, declared)| ComputedStep {
                    declared,
                    statistic: computed.martingale,
                    p_value_ppm: computed.p_value_ppm,
                    stop: computed.stop,
                })
                .collect(),
            if replay.stopped {
                AuditAlgorithmDecision::Pass
            } else {
                AuditAlgorithmDecision::Continue
            },
        ),
        Err(err) => boundary_algorithm_transcript(run, &err.to_string()),
    }
}

fn replay_minerva_run(run: &AuditAlgorithmRun) -> AlgorithmReplayTranscript {
    let Some(risk_limit_ppm) = run.risk_limit_ppm else {
        return boundary_algorithm_transcript(run, "Minerva replay requires risk_limit_ppm");
    };
    let Some(winner_votes) = run.reported_winner_votes else {
        return boundary_algorithm_transcript(run, "Minerva replay requires reported_winner_votes");
    };
    let Some(loser_votes) = run.reported_loser_votes else {
        return boundary_algorithm_transcript(run, "Minerva replay requires reported_loser_votes");
    };

    if run.sample_steps.is_empty() {
        return boundary_algorithm_transcript(
            run,
            "Minerva round-one replay requires at least one sample step",
        );
    }

    let rounds = match minerva_rounds_from_steps(run) {
        Ok(rounds) => rounds,
        Err(err) => return boundary_algorithm_transcript(run, &err),
    };
    let declared_round_steps = minerva_declared_round_steps(run);

    match replay_minerva_ballot_polling_rounds(winner_votes, loser_votes, risk_limit_ppm, &rounds) {
        Ok(replay) => build_replay_transcript(
            run,
            replay
                .steps
                .into_iter()
                .zip(declared_round_steps.iter())
                .map(|(computed, declared)| ComputedStep {
                    declared,
                    statistic: computed.likelihood_ratio,
                    p_value_ppm: computed.p_value_ppm,
                    stop: computed.stop,
                })
                .collect(),
            if replay.stopped {
                AuditAlgorithmDecision::Pass
            } else {
                AuditAlgorithmDecision::Continue
            },
        ),
        Err(err) => boundary_algorithm_transcript(run, &err.to_string()),
    }
}

fn minerva_rounds_from_steps(
    run: &AuditAlgorithmRun,
) -> Result<Vec<MinervaRoundObservationSet>, String> {
    let uses_explicit_rounds = run
        .sample_steps
        .iter()
        .any(|step| step.round_index.is_some());
    if !uses_explicit_rounds {
        let observations = run
            .sample_steps
            .iter()
            .map(bravo_observation_from_step)
            .collect::<Result<Vec<_>, _>>()
            .map_err(str::to_string)?;
        return Ok(vec![MinervaRoundObservationSet {
            round_index: 0,
            observations,
        }]);
    }

    let mut rounds = Vec::new();
    let mut current_round_index = None;
    let mut current_observations = Vec::new();
    for step in &run.sample_steps {
        let Some(round_index) = step.round_index else {
            return Err(
                "Minerva explicit round replay requires round_index on every sample step"
                    .to_string(),
            );
        };
        if current_round_index.is_some_and(|current| round_index < current) {
            return Err(
                "Minerva sample steps must be ordered by nondecreasing round_index".to_string(),
            );
        }
        if current_round_index.is_some_and(|current| round_index != current) {
            rounds.push(MinervaRoundObservationSet {
                round_index: current_round_index.unwrap(),
                observations: std::mem::take(&mut current_observations),
            });
        }
        current_round_index = Some(round_index);
        current_observations.push(bravo_observation_from_step(step).map_err(str::to_string)?);
    }

    if let Some(round_index) = current_round_index {
        rounds.push(MinervaRoundObservationSet {
            round_index,
            observations: current_observations,
        });
    }
    Ok(rounds)
}

fn minerva_declared_round_steps(run: &AuditAlgorithmRun) -> Vec<&rcount_core::AuditSampleStep> {
    let uses_explicit_rounds = run
        .sample_steps
        .iter()
        .any(|step| step.round_index.is_some());
    if !uses_explicit_rounds {
        return run.sample_steps.last().into_iter().collect();
    }

    run.sample_steps
        .iter()
        .enumerate()
        .filter_map(|(index, step)| {
            let next_round = run
                .sample_steps
                .get(index + 1)
                .and_then(|next_step| next_step.round_index);
            if next_round != step.round_index {
                Some(step)
            } else {
                None
            }
        })
        .collect()
}

fn replay_comparison_taint_product_run(
    run: &AuditAlgorithmRun,
    missing_risk_boundary: &str,
) -> AlgorithmReplayTranscript {
    let Some(risk_limit_ppm) = run.risk_limit_ppm else {
        return boundary_algorithm_transcript(run, missing_risk_boundary);
    };

    let taints = run
        .sample_steps
        .iter()
        .map(|step| rational_from_core(step.assorter_value).map_err(|err| err.to_string()))
        .collect::<Result<Vec<_>, _>>();
    let taints = match taints {
        Ok(taints) => taints,
        Err(err) => return boundary_algorithm_transcript(run, &err),
    };

    match replay_kaplan_markov_taint_product(risk_limit_ppm, &taints) {
        Ok(replay) => build_replay_transcript(
            run,
            replay
                .steps
                .into_iter()
                .zip(run.sample_steps.iter())
                .map(|(computed, declared)| ComputedStep {
                    declared,
                    statistic: computed.p_value,
                    p_value_ppm: computed.p_value_ppm,
                    stop: computed.stop,
                })
                .collect(),
            if replay.stopped {
                AuditAlgorithmDecision::Pass
            } else {
                AuditAlgorithmDecision::Continue
            },
        ),
        Err(err) => boundary_algorithm_transcript(run, &err.to_string()),
    }
}

fn replay_kaplan_markov_run(run: &AuditAlgorithmRun) -> AlgorithmReplayTranscript {
    match (
        run.macro_ballot_count,
        run.macro_reported_margin,
        run.macro_gamma,
    ) {
        (Some(ballot_count), Some(reported_margin), Some(gamma)) => {
            replay_kaplan_markov_macro_run(run, ballot_count, reported_margin, gamma)
        }
        (None, None, None) => replay_comparison_taint_product_run(
            run,
            "Kaplan-Markov taint-product replay requires risk_limit_ppm",
        ),
        _ => boundary_algorithm_transcript(
            run,
            "Kaplan-Markov MACRO replay requires macro_ballot_count, macro_reported_margin, and macro_gamma together",
        ),
    }
}

fn replay_kaplan_markov_macro_run(
    run: &AuditAlgorithmRun,
    ballot_count: u64,
    reported_margin: u64,
    gamma: RationalValue,
) -> AlgorithmReplayTranscript {
    let Some(risk_limit_ppm) = run.risk_limit_ppm else {
        return boundary_algorithm_transcript(
            run,
            "Kaplan-Markov MACRO replay requires risk_limit_ppm",
        );
    };
    let gamma = match rational_from_core(gamma) {
        Ok(value) => value,
        Err(err) => return boundary_algorithm_transcript(run, &err.to_string()),
    };
    let overstatements = run
        .sample_steps
        .iter()
        .map(|step| {
            if step.assorter_value.denominator != 1 {
                return Err(
                    "Kaplan-Markov MACRO overstatement steps must be integer categories"
                        .to_string(),
                );
            }
            i8::try_from(step.assorter_value.numerator).map_err(|_| {
                "Kaplan-Markov MACRO overstatement category is out of range".to_string()
            })
        })
        .collect::<Result<Vec<_>, _>>();
    let overstatements = match overstatements {
        Ok(value) => value,
        Err(err) => return boundary_algorithm_transcript(run, &err),
    };

    match replay_kaplan_markov_macro_bound(
        KaplanMarkovMacroConfig {
            ballot_count,
            reported_margin,
            gamma,
            risk_limit_ppm,
        },
        &overstatements,
    ) {
        Ok(replay) => build_replay_transcript(
            run,
            replay
                .steps
                .into_iter()
                .zip(run.sample_steps.iter())
                .map(|(computed, declared)| ComputedStep {
                    declared,
                    statistic: computed.p_value,
                    p_value_ppm: computed.p_value_ppm,
                    stop: computed.stop,
                })
                .collect(),
            if replay.stopped {
                AuditAlgorithmDecision::Pass
            } else {
                AuditAlgorithmDecision::Continue
            },
        ),
        Err(err) => boundary_algorithm_transcript(run, &err.to_string()),
    }
}

struct ComputedStep<'a> {
    declared: &'a rcount_core::AuditSampleStep,
    statistic: Rational,
    p_value_ppm: u32,
    stop: bool,
}

fn build_replay_transcript(
    run: &AuditAlgorithmRun,
    computed_steps: Vec<ComputedStep<'_>>,
    computed_decision: AuditAlgorithmDecision,
) -> AlgorithmReplayTranscript {
    let mut failed = run.decision != computed_decision;
    let steps = computed_steps
        .into_iter()
        .map(|computed| {
            let statistic = rational_to_core(computed.statistic);
            let mut errors = Vec::new();
            if computed
                .declared
                .statistic
                .is_some_and(|declared| declared != statistic)
            {
                errors.push("declared statistic mismatch");
            }
            if computed
                .declared
                .p_value_ppm
                .is_some_and(|declared| declared != computed.p_value_ppm)
            {
                errors.push("declared p-value mismatch");
            }
            failed |= !errors.is_empty();
            AlgorithmReplayStep {
                step_index: computed.declared.step_index,
                assertion_id: computed.declared.assertion_id.clone(),
                sample_unit_id: computed.declared.sample_unit_id.clone(),
                statistic,
                p_value_ppm: computed.p_value_ppm,
                stop: computed.stop,
                error: if errors.is_empty() {
                    None
                } else {
                    Some(errors.join("; "))
                },
            }
        })
        .collect();

    AlgorithmReplayTranscript {
        run_id: run.run_id.clone(),
        method_id: run.method_id.clone(),
        status: if failed {
            AlgorithmReplayStatus::Fail
        } else {
            AlgorithmReplayStatus::Pass
        },
        decision: run.decision,
        computed_decision: Some(computed_decision),
        steps,
        boundary: None,
    }
}

fn boundary_algorithm_transcript(
    run: &AuditAlgorithmRun,
    boundary: &str,
) -> AlgorithmReplayTranscript {
    AlgorithmReplayTranscript {
        run_id: run.run_id.clone(),
        method_id: run.method_id.clone(),
        status: AlgorithmReplayStatus::Boundary,
        decision: run.decision,
        computed_decision: None,
        steps: Vec::new(),
        boundary: Some(boundary.to_string()),
    }
}

fn bravo_observation_from_step(
    step: &rcount_core::AuditSampleStep,
) -> Result<BravoObservation, &'static str> {
    match (
        step.assorter_value.numerator,
        step.assorter_value.denominator,
    ) {
        (1, 1) => Ok(BravoObservation::Winner),
        (0, 1) => Ok(BravoObservation::Loser),
        (_, denominator) if denominator > 0 => Ok(BravoObservation::Other),
        _ => Err("BRAVO observation has invalid rational value"),
    }
}

fn rational_from_core(value: RationalValue) -> Result<Rational, RcountStatsError> {
    Rational::new(value.numerator as i128, value.denominator as i128)
}

fn rational_to_core(value: Rational) -> RationalValue {
    RationalValue {
        numerator: value.numerator as i64,
        denominator: value.denominator as i64,
    }
}

pub fn verify_and_write_transcript(dir: &Path) -> Result<VerificationTranscript, RcountAuditError> {
    let transcript = verify_package_dir(dir);
    write_verification_transcript(dir, &transcript)?;
    Ok(transcript)
}

fn verify_loaded_package(
    dir: &Path,
    manifest: &RcountManifest,
    package: &RcountPackage,
) -> VerificationTranscript {
    let package_hash = package_content_hash(package).unwrap_or_else(|err| format!("error:{err}"));
    let mut checks = Vec::new();

    match verify_package(package) {
        Ok(report) => {
            checks.extend(report.passed.into_iter().map(pass_result));
        }
        Err(err) => checks.push(CheckResult {
            equation_id: equation_id_for_core_error(&err).to_string(),
            status: VerificationStatus::Fail,
            contest_id: None,
            reporting_unit_id: None,
            error: Some(err.to_string()),
        }),
    }

    for contest in &package.contests {
        for jurisdiction_unit in package
            .reporting_units
            .iter()
            .filter(|unit| unit.kind == ReportingUnitKind::JurisdictionTotal)
        {
            match verify_jurisdiction_total(
                &contest.contest_id,
                &jurisdiction_unit.reporting_unit_id,
                &package.summaries,
            ) {
                Ok(passes) => {
                    checks.extend(passes.into_iter().map(pass_result));
                }
                Err(err) => checks.push(CheckResult {
                    equation_id: "jurisdiction_contest_total".to_string(),
                    status: VerificationStatus::Fail,
                    contest_id: Some(contest.contest_id.clone()),
                    reporting_unit_id: Some(jurisdiction_unit.reporting_unit_id.clone()),
                    error: Some(err.to_string()),
                }),
            }
        }
    }

    if package
        .status_events
        .iter()
        .any(|event| event.event_type == StatusEventType::Correction)
    {
        match verify_canvass_correction_event(package) {
            Ok(pass) => checks.push(pass_result(pass)),
            Err(err) => checks.push(CheckResult {
                equation_id: "canvass_correction_event".to_string(),
                status: VerificationStatus::Fail,
                contest_id: None,
                reporting_unit_id: None,
                error: Some(err.to_string()),
            }),
        }
    }

    match verify_source_index(dir) {
        Ok(source_checks) => {
            checks.extend(source_checks.into_iter().map(|source| CheckResult {
                equation_id: "source_hash_match".to_string(),
                status: VerificationStatus::Pass,
                contest_id: None,
                reporting_unit_id: Some(source.source_id),
                error: None,
            }));
        }
        Err(err) => checks.push(CheckResult {
            equation_id: "source_hash_match".to_string(),
            status: VerificationStatus::Fail,
            contest_id: None,
            reporting_unit_id: None,
            error: Some(err.to_string()),
        }),
    }

    let status = if checks
        .iter()
        .all(|check| check.status == VerificationStatus::Pass)
    {
        VerificationStatus::Pass
    } else {
        VerificationStatus::Fail
    };

    VerificationTranscript {
        transcript_version: RCOUNT_AUDIT_TRANSCRIPT_VERSION.to_string(),
        verifier: "rcount-audit".to_string(),
        status,
        package_content_hash: package_hash,
        manifest_content_hash: manifest.content_hash.clone(),
        checks,
    }
}

fn pass_result(pass: EquationPass) -> CheckResult {
    CheckResult {
        equation_id: pass.equation_id,
        status: VerificationStatus::Pass,
        contest_id: Some(pass.contest_id),
        reporting_unit_id: Some(pass.reporting_unit_id),
        error: None,
    }
}

fn equation_id_for_core_error(err: &RcountCoreError) -> &'static str {
    match err {
        RcountCoreError::MissingBatch { .. }
        | RcountCoreError::DuplicateBatchId { .. }
        | RcountCoreError::BatchSummaryTotalMismatch { .. } => "batch_summary_total",
        RcountCoreError::AcceptedBallotsMismatch { .. } => "accepted_ballots",
        RcountCoreError::DuplicateLineageId { .. }
        | RcountCoreError::MissingPriorLineageUnit { .. }
        | RcountCoreError::MissingCurrentLineageUnit { .. }
        | RcountCoreError::InvalidSplitLineage { .. }
        | RcountCoreError::InvalidMergeLineage { .. } => "lineage_conservation",
        RcountCoreError::DuplicateRhistReference { .. }
        | RcountCoreError::InvalidRhistPackageHash { .. }
        | RcountCoreError::EmptyRhistCycleRefs { .. }
        | RcountCoreError::UnsupportedRhistReferenceRole { .. } => "rhist_reference_declared",
        RcountCoreError::DuplicateRctxReference { .. }
        | RcountCoreError::InvalidRctxContextHash { .. }
        | RcountCoreError::InvalidRctxCrosswalkHash { .. }
        | RcountCoreError::UnsupportedRctxReferenceRole { .. } => "rctx_reference_declared",
        RcountCoreError::DuplicateProofId { .. }
        | RcountCoreError::ChoiceBearingProof { .. }
        | RcountCoreError::LinkableVoterProof { .. }
        | RcountCoreError::InvalidProofTokenHash { .. } => "proof_privacy_gate",
        RcountCoreError::DuplicateCvrContest { .. }
        | RcountCoreError::InvalidCvrContestCardinality { .. }
        | RcountCoreError::UnknownCvrSelection { .. }
        | RcountCoreError::MissingCvrSummary { .. }
        | RcountCoreError::CvrSummaryMismatch { .. } => "cvr_summary_total",
        RcountCoreError::DuplicateAuditAlgorithmRunId { .. }
        | RcountCoreError::InvalidAuditAlgorithmRiskLimit { .. }
        | RcountCoreError::InvalidAuditMacroDesign { .. }
        | RcountCoreError::InvalidStratifiedHybridDesign { .. }
        | RcountCoreError::MissingStratifiedHybridComponent { .. }
        | RcountCoreError::InvalidRankedChoiceAuditDesign { .. }
        | RcountCoreError::InvalidRankedChoiceSample { .. }
        | RcountCoreError::InvalidBayesianAuditDesign { .. }
        | RcountCoreError::InvalidObservableBallotAuditDesign { .. }
        | RcountCoreError::MissingObservableBallotOpening { .. }
        | RcountCoreError::UnsupportedAuditAlgorithmMethod { .. }
        | RcountCoreError::DuplicateAuditAssertion { .. }
        | RcountCoreError::InvalidAuditAssorterBound { .. }
        | RcountCoreError::MissingAuditAssertion { .. }
        | RcountCoreError::DuplicateAuditSampleStep { .. }
        | RcountCoreError::InvalidAuditAssorterValue { .. }
        | RcountCoreError::InvalidAuditPValue { .. } => "audit_algorithm_transcript",
        RcountCoreError::DuplicateRlaAuditId { .. }
        | RcountCoreError::InvalidRlaRiskLimit { .. }
        | RcountCoreError::InvalidRlaSampleSize { .. }
        | RcountCoreError::UnsupportedRlaSamplingAlgorithm { .. }
        | RcountCoreError::MissingRlaPopulation { .. }
        | RcountCoreError::RlaManifestHashMismatch { .. }
        | RcountCoreError::RlaSampleMismatch { .. } => "rla_sampler_replay",
        RcountCoreError::MissingRlaStoppingRule { .. }
        | RcountCoreError::DuplicateRlaObservation { .. }
        | RcountCoreError::MissingRlaObservation { .. }
        | RcountCoreError::RlaObservationCvrMismatch { .. }
        | RcountCoreError::RlaStoppingStatusMismatch { .. }
        | RcountCoreError::RlaDiscrepancyCountMismatch { .. }
        | RcountCoreError::RlaDiscrepancyMismatch { .. }
        | RcountCoreError::MissingRlaRiskEstimate { .. }
        | RcountCoreError::RlaRiskEstimateMismatch { .. } => "rla_stopping_rule",
        RcountCoreError::MissingRlaMarginMetadata { .. }
        | RcountCoreError::MissingRlaMarginSelection { .. }
        | RcountCoreError::InvalidRlaReportedMargin { .. }
        | RcountCoreError::RlaWinnerVotesMismatch { .. }
        | RcountCoreError::RlaLoserVotesMismatch { .. }
        | RcountCoreError::RlaReportedMarginMismatch { .. }
        | RcountCoreError::RlaDilutedMarginDenominatorMismatch { .. } => "rla_margin_metadata",
        RcountCoreError::UnsupportedRlaJurisdictionMethod { .. }
        | RcountCoreError::InvalidColoradoRlaSeed { .. }
        | RcountCoreError::MissingColoradoRlaComparisonFields { .. }
        | RcountCoreError::MissingCaliforniaRlaPublicToolFields { .. }
        | RcountCoreError::InvalidCaliforniaRlaManifestFormat { .. }
        | RcountCoreError::InvalidRlaSoftwareSourceUrl { .. } => "rla_jurisdiction_adapter",
        RcountCoreError::DuplicateStatusEventId { .. }
        | RcountCoreError::NoStatusTransition { .. }
        | RcountCoreError::IncompleteStatusEvent { .. } => "status_event_declared",
        RcountCoreError::DuplicateManualAuditId { .. }
        | RcountCoreError::MissingManualAuditSummary { .. }
        | RcountCoreError::ManualAuditMachineTotalMismatch { .. }
        | RcountCoreError::ManualAuditStatusMismatch { .. } => "manual_audit_reconciliation",
        RcountCoreError::DuplicateBatchComparisonAuditId { .. }
        | RcountCoreError::MissingBatchComparisonBatch { .. }
        | RcountCoreError::BatchComparisonBatchSizeMismatch { .. }
        | RcountCoreError::MissingBatchComparisonSummary { .. }
        | RcountCoreError::BatchComparisonReportedTotalMismatch { .. }
        | RcountCoreError::MissingBatchComparisonHandTally { .. }
        | RcountCoreError::BatchComparisonReportedMarginMismatch { .. }
        | RcountCoreError::BatchComparisonHandMarginMismatch { .. }
        | RcountCoreError::BatchComparisonOverstatementMismatch { .. } => {
            "batch_comparison_overstatement"
        }
        RcountCoreError::MissingBatchComparisonAlgorithmEvidence { .. }
        | RcountCoreError::BatchComparisonAlgorithmTaintMismatch { .. }
        | RcountCoreError::EmptyBatchComparisonAlgorithmSample { .. }
        | RcountCoreError::InvalidBatchComparisonAlgorithmMargin { .. }
        | RcountCoreError::BatchComparisonAlgorithmAssertionMismatch { .. } => {
            "batch_comparison_algorithm_linkage"
        }
        RcountCoreError::MissingCanvassCorrectionEvent
        | RcountCoreError::MissingStatusSummaries { .. } => "canvass_correction_event",
        _ => "contest_selection_sum",
    }
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
        synthetic_batch_comparison_package, synthetic_bayesian_tabulation_boundary_package,
        synthetic_california_rla_package, synthetic_canvass_correction_package,
        synthetic_choice_bearing_proof_package, synthetic_colorado_rla_package,
        synthetic_cvr_summary_package, synthetic_mail_batch_added_package,
        synthetic_manual_audit_package, synthetic_minerva_multi_round_package,
        synthetic_minerva_round_one_package, synthetic_missing_batch_package,
        synthetic_precinct_split_lineage_package, synthetic_privacy_inclusion_package,
        synthetic_raire_boundary_package, synthetic_rla_discrepancy_package,
        synthetic_rla_margin_package, synthetic_rla_replay_package,
        synthetic_rla_statistical_package, synthetic_rla_stopping_package,
        synthetic_soba_observable_ballot_boundary_package, synthetic_stratified_hybrid_package,
        synthetic_summary_basic_package,
    };
    use rcount_io::{
        synthetic_canvass_correction_manifest, synthetic_summary_basic_manifest, write_package_dir,
    };

    #[test]
    fn valid_summary_basic_produces_pass_transcript() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_summary_basic_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();

        let transcript = verify_package_dir(tmp.path());
        assert_eq!(transcript.status, VerificationStatus::Pass);
        assert_eq!(transcript.checks.len(), 5);
        assert_eq!(
            transcript.package_content_hash,
            transcript.manifest_content_hash
        );
    }

    #[test]
    fn tampered_manifest_produces_fail_transcript() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_summary_basic_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();

        let manifest_path = tmp.path().join("manifest.json");
        let mut raw: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&manifest_path).unwrap()).unwrap();
        raw["content_hash"] = serde_json::Value::String("sha256:bad".to_string());
        std::fs::write(&manifest_path, serde_json::to_vec_pretty(&raw).unwrap()).unwrap();

        let transcript = verify_package_dir(tmp.path());
        assert_eq!(transcript.status, VerificationStatus::Fail);
        assert_eq!(transcript.checks[0].equation_id, "package_read");
        assert!(transcript.checks[0]
            .error
            .as_ref()
            .unwrap()
            .contains("content_hash mismatch"));
    }

    #[test]
    fn bad_arithmetic_produces_fail_transcript() {
        let tmp = tempfile::tempdir().unwrap();
        let mut package = synthetic_summary_basic_package();
        package.summaries[0].counted_ballots += 1;
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();

        let transcript = verify_package_dir(tmp.path());
        assert_eq!(transcript.status, VerificationStatus::Fail);
        assert!(transcript
            .checks
            .iter()
            .any(|check| check.equation_id == "contest_selection_sum"
                && check.status == VerificationStatus::Fail));
    }

    #[test]
    fn tampered_source_produces_fail_transcript() {
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

        let transcript = verify_package_dir(tmp.path());
        assert_eq!(transcript.status, VerificationStatus::Fail);
        assert!(transcript
            .checks
            .iter()
            .any(|check| check.equation_id == "source_hash_match"
                && check.status == VerificationStatus::Fail));
    }

    #[test]
    fn missing_source_hash_produces_fail_transcript() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_summary_basic_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();
        std::fs::write(
            tmp.path().join("sources").join("source-index.json"),
            br#"{"sources":[]}"#,
        )
        .unwrap();

        let transcript = verify_package_dir(tmp.path());
        assert_eq!(transcript.status, VerificationStatus::Fail);
        assert!(transcript
            .checks
            .iter()
            .any(|check| check.equation_id == "source_hash_match"
                && check
                    .error
                    .as_deref()
                    .is_some_and(|error| error.contains("source index is empty"))));
    }

    #[test]
    fn canvass_correction_produces_event_correlation_pass() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_canvass_correction_package();
        let manifest = synthetic_canvass_correction_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();

        let transcript = verify_package_dir(tmp.path());
        assert_eq!(transcript.status, VerificationStatus::Pass);
        assert!(transcript
            .checks
            .iter()
            .any(|check| check.equation_id == "canvass_correction_event"
                && check.status == VerificationStatus::Pass));
    }

    #[test]
    fn mail_batch_added_produces_batch_correlation_passes() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_mail_batch_added_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();

        let transcript = verify_package_dir(tmp.path());
        assert_eq!(transcript.status, VerificationStatus::Pass);
        assert_eq!(
            transcript
                .checks
                .iter()
                .filter(|check| check.equation_id == "batch_summary_total"
                    && check.status == VerificationStatus::Pass)
                .count(),
            3
        );
    }

    #[test]
    fn missing_batch_produces_batch_correlation_failure() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_missing_batch_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();

        let transcript = verify_package_dir(tmp.path());
        assert_eq!(transcript.status, VerificationStatus::Fail);
        assert!(transcript
            .checks
            .iter()
            .any(|check| check.equation_id == "batch_summary_total"
                && check
                    .error
                    .as_deref()
                    .is_some_and(|error| error.contains("references missing batch id"))));
    }

    #[test]
    fn precinct_split_lineage_produces_lineage_passes() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_precinct_split_lineage_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();

        let transcript = verify_package_dir(tmp.path());
        assert_eq!(transcript.status, VerificationStatus::Pass);
        assert_eq!(
            transcript
                .checks
                .iter()
                .filter(|check| check.equation_id == "lineage_conservation"
                    && check.status == VerificationStatus::Pass)
                .count(),
            2
        );
    }

    #[test]
    fn bad_lineage_produces_lineage_failure() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_bad_lineage_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();

        let transcript = verify_package_dir(tmp.path());
        assert_eq!(transcript.status, VerificationStatus::Fail);
        assert!(transcript
            .checks
            .iter()
            .any(|check| check.equation_id == "lineage_conservation"
                && check
                    .error
                    .as_deref()
                    .is_some_and(|error| error.contains("missing current reporting unit"))));
    }

    #[test]
    fn privacy_inclusion_produces_privacy_gate_pass() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_privacy_inclusion_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();

        let transcript = verify_package_dir(tmp.path());
        assert_eq!(transcript.status, VerificationStatus::Pass);
        assert!(transcript
            .checks
            .iter()
            .any(|check| check.equation_id == "proof_privacy_gate"
                && check.status == VerificationStatus::Pass));
    }

    #[test]
    fn choice_bearing_proof_produces_privacy_gate_failure() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_choice_bearing_proof_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();

        let transcript = verify_package_dir(tmp.path());
        assert_eq!(transcript.status, VerificationStatus::Fail);
        assert!(transcript
            .checks
            .iter()
            .any(|check| check.equation_id == "proof_privacy_gate"
                && check
                    .error
                    .as_deref()
                    .is_some_and(|error| error.contains("exposes candidate selections"))));
    }

    #[test]
    fn cvr_summary_package_produces_cvr_reconciliation_passes() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_cvr_summary_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();

        let transcript = verify_package_dir(tmp.path());
        assert_eq!(transcript.status, VerificationStatus::Pass);
        assert_eq!(
            transcript
                .checks
                .iter()
                .filter(|check| check.equation_id == "cvr_summary_total"
                    && check.status == VerificationStatus::Pass)
                .count(),
            2
        );
    }

    #[test]
    fn bad_cvr_summary_package_produces_cvr_reconciliation_failure() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_bad_cvr_summary_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();

        let transcript = verify_package_dir(tmp.path());
        assert_eq!(transcript.status, VerificationStatus::Fail);
        assert!(transcript
            .checks
            .iter()
            .any(|check| check.equation_id == "cvr_summary_total"
                && check
                    .error
                    .as_deref()
                    .is_some_and(|error| error.contains("CVR summary mismatch"))));
    }

    #[test]
    fn rla_replay_package_produces_sampler_replay_pass() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_rla_replay_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();

        let transcript = verify_package_dir(tmp.path());
        assert_eq!(transcript.status, VerificationStatus::Pass);
        assert!(transcript
            .checks
            .iter()
            .any(|check| check.equation_id == "rla_sampler_replay"
                && check.status == VerificationStatus::Pass));
    }

    #[test]
    fn bad_rla_replay_package_produces_sampler_replay_failure() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_bad_rla_replay_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();

        let transcript = verify_package_dir(tmp.path());
        assert_eq!(transcript.status, VerificationStatus::Fail);
        assert!(transcript
            .checks
            .iter()
            .any(|check| check.equation_id == "rla_sampler_replay"
                && check
                    .error
                    .as_deref()
                    .is_some_and(|error| error.contains("sample mismatch"))));
    }

    #[test]
    fn rla_stopping_package_produces_stopping_rule_pass() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_rla_stopping_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();

        let transcript = verify_package_dir(tmp.path());
        assert_eq!(transcript.status, VerificationStatus::Pass);
        assert!(transcript
            .checks
            .iter()
            .any(|check| check.equation_id == "rla_stopping_rule"
                && check.status == VerificationStatus::Pass));
    }

    #[test]
    fn bad_rla_stopping_package_produces_stopping_rule_failure() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_bad_rla_stopping_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();

        let transcript = verify_package_dir(tmp.path());
        assert_eq!(transcript.status, VerificationStatus::Fail);
        assert!(transcript
            .checks
            .iter()
            .any(|check| check.equation_id == "rla_stopping_rule"
                && check
                    .error
                    .as_deref()
                    .is_some_and(|error| error.contains("computed Escalate"))));
    }

    #[test]
    fn rla_discrepancy_package_produces_stopping_rule_pass() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_rla_discrepancy_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();

        let transcript = verify_package_dir(tmp.path());
        assert_eq!(transcript.status, VerificationStatus::Pass);
        assert!(transcript
            .checks
            .iter()
            .any(|check| check.equation_id == "rla_stopping_rule"
                && check.status == VerificationStatus::Pass));
    }

    #[test]
    fn bad_rla_discrepancy_package_produces_taxonomy_failure() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_bad_rla_discrepancy_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();

        let transcript = verify_package_dir(tmp.path());
        assert_eq!(transcript.status, VerificationStatus::Fail);
        assert!(transcript
            .checks
            .iter()
            .any(|check| check.equation_id == "rla_stopping_rule"
                && check
                    .error
                    .as_deref()
                    .is_some_and(|error| error.contains("discrepancy mismatch"))));
    }

    #[test]
    fn rla_margin_package_produces_margin_metadata_pass() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_rla_margin_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();

        let transcript = verify_package_dir(tmp.path());
        assert_eq!(transcript.status, VerificationStatus::Pass);
        assert!(transcript
            .checks
            .iter()
            .any(|check| check.equation_id == "rla_margin_metadata"
                && check.status == VerificationStatus::Pass));
    }

    #[test]
    fn bad_rla_margin_package_produces_margin_metadata_failure() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_bad_rla_margin_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();

        let transcript = verify_package_dir(tmp.path());
        assert_eq!(transcript.status, VerificationStatus::Fail);
        assert!(transcript
            .checks
            .iter()
            .any(|check| check.equation_id == "rla_margin_metadata"
                && check
                    .error
                    .as_deref()
                    .is_some_and(|error| error.contains("reported margin mismatch"))));
    }

    #[test]
    fn rla_statistical_package_produces_stopping_rule_pass() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_rla_statistical_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();

        let transcript = verify_package_dir(tmp.path());
        assert_eq!(transcript.status, VerificationStatus::Pass);
        assert!(transcript
            .checks
            .iter()
            .any(|check| check.equation_id == "rla_stopping_rule"
                && check.status == VerificationStatus::Pass));
    }

    #[test]
    fn bad_rla_statistical_package_produces_risk_failure() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_bad_rla_statistical_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();

        let transcript = verify_package_dir(tmp.path());
        assert_eq!(transcript.status, VerificationStatus::Fail);
        assert!(transcript
            .checks
            .iter()
            .any(|check| check.equation_id == "rla_stopping_rule"
                && check
                    .error
                    .as_deref()
                    .is_some_and(|error| error.contains("risk estimate mismatch"))));
    }

    #[test]
    fn colorado_rla_package_produces_jurisdiction_adapter_pass() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_colorado_rla_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();

        let transcript = verify_package_dir(tmp.path());
        assert_eq!(transcript.status, VerificationStatus::Pass);
        assert!(transcript
            .checks
            .iter()
            .any(|check| check.equation_id == "rla_jurisdiction_adapter"
                && check.status == VerificationStatus::Pass));
    }

    #[test]
    fn bad_colorado_rla_package_produces_seed_failure() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_bad_colorado_rla_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();

        let transcript = verify_package_dir(tmp.path());
        assert_eq!(transcript.status, VerificationStatus::Fail);
        assert!(transcript.checks.iter().any(|check| check.equation_id
            == "rla_jurisdiction_adapter"
            && check
                .error
                .as_deref()
                .is_some_and(|error| error.contains("invalid Colorado-style public seed"))));
    }

    #[test]
    fn california_rla_package_produces_jurisdiction_adapter_pass() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_california_rla_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();

        let transcript = verify_package_dir(tmp.path());
        assert_eq!(transcript.status, VerificationStatus::Pass);
        assert!(transcript
            .checks
            .iter()
            .any(|check| check.equation_id == "rla_jurisdiction_adapter"
                && check.status == VerificationStatus::Pass));
    }

    #[test]
    fn bad_california_rla_package_produces_source_url_failure() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_bad_california_rla_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();

        let transcript = verify_package_dir(tmp.path());
        assert_eq!(transcript.status, VerificationStatus::Fail);
        assert!(transcript.checks.iter().any(|check| check.equation_id
            == "rla_jurisdiction_adapter"
            && check
                .error
                .as_deref()
                .is_some_and(|error| error.contains("invalid public audit software source URL"))));
    }

    #[test]
    fn manual_audit_package_produces_reconciliation_pass() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_manual_audit_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();

        let transcript = verify_package_dir(tmp.path());
        assert_eq!(transcript.status, VerificationStatus::Pass);
        assert!(transcript
            .checks
            .iter()
            .any(|check| check.equation_id == "manual_audit_reconciliation"
                && check.status == VerificationStatus::Pass));
    }

    #[test]
    fn batch_comparison_package_produces_overstatement_pass() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_batch_comparison_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();

        let transcript = verify_package_dir(tmp.path());
        assert_eq!(transcript.status, VerificationStatus::Pass);
        assert!(transcript.checks.iter().any(|check| check.equation_id
            == "batch_comparison_overstatement"
            && check.status == VerificationStatus::Pass));
    }

    #[test]
    fn bad_manual_audit_package_produces_reconciliation_failure() {
        let tmp = tempfile::tempdir().unwrap();
        let package = synthetic_bad_manual_audit_package();
        let manifest = synthetic_summary_basic_manifest(&package).unwrap();
        write_package_dir(tmp.path(), &manifest, &package).unwrap();

        let transcript = verify_package_dir(tmp.path());
        assert_eq!(transcript.status, VerificationStatus::Fail);
        assert!(transcript.checks.iter().any(|check| check.equation_id
            == "manual_audit_reconciliation"
            && check
                .error
                .as_deref()
                .is_some_and(|error| error.contains("declares status Pass, computed Escalate"))));
    }

    #[test]
    fn docs_summary_basic_transcript_verifies_when_present() {
        let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("docs")
            .join("examples")
            .join("rcount-golden-packages")
            .join("summary-basic");
        if dir.exists() {
            let transcript = verify_package_dir(&dir);
            assert_eq!(transcript.status, VerificationStatus::Pass);
        }
    }

    #[test]
    fn bravo_algorithm_statistics_replay_passes_toy_run() {
        let run = AuditAlgorithmRun {
            run_id: "audit-run:bravo-toy".to_string(),
            contest_id: "syn-2024-mayor".to_string(),
            method_id: BRAVO_BALLOT_POLLING_METHOD_ID.to_string(),
            sampling_mode: rcount_core::AuditSamplingMode::WithReplacement,
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
            assertions: vec![rcount_core::AuditAssertion {
                assertion_id: "assertion:cand-a-over-cand-b".to_string(),
                kind: rcount_core::AuditAssertionKind::PluralityWinnerLoser,
                assorter_id: "plurality-winner-loser-v1".to_string(),
                assorter_upper_bound: RationalValue {
                    numerator: 1,
                    denominator: 1,
                },
                winner_selection_id: Some("cand-a".to_string()),
                loser_selection_id: Some("cand-b".to_string()),
            }],
            sample_steps: (0..6)
                .map(|step_index| rcount_core::AuditSampleStep {
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
        };

        let transcript = replay_audit_algorithm_statistics(&run);
        assert_eq!(transcript.status, AlgorithmReplayStatus::Pass);
        assert_eq!(
            transcript.computed_decision,
            Some(AuditAlgorithmDecision::Pass)
        );
        assert_eq!(transcript.steps.last().unwrap().p_value_ppm, 87_792);
    }

    #[test]
    fn bravo_algorithm_statistics_replay_continues_when_threshold_is_not_met() {
        let run = AuditAlgorithmRun {
            run_id: "audit-run:bravo-continue".to_string(),
            contest_id: "syn-2024-mayor".to_string(),
            method_id: BRAVO_BALLOT_POLLING_METHOD_ID.to_string(),
            sampling_mode: rcount_core::AuditSamplingMode::WithReplacement,
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
            assertions: vec![rcount_core::AuditAssertion {
                assertion_id: "assertion:cand-a-over-cand-b".to_string(),
                kind: rcount_core::AuditAssertionKind::PluralityWinnerLoser,
                assorter_id: "plurality-winner-loser-v1".to_string(),
                assorter_upper_bound: RationalValue {
                    numerator: 1,
                    denominator: 1,
                },
                winner_selection_id: Some("cand-a".to_string()),
                loser_selection_id: Some("cand-b".to_string()),
            }],
            sample_steps: vec![rcount_core::AuditSampleStep {
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
        };

        let transcript = replay_audit_algorithm_statistics(&run);
        assert_eq!(transcript.status, AlgorithmReplayStatus::Pass);
        assert_eq!(
            transcript.computed_decision,
            Some(AuditAlgorithmDecision::Continue)
        );
        assert_eq!(
            transcript.steps[0].statistic,
            RationalValue {
                numerator: 3,
                denominator: 2
            }
        );
    }

    #[test]
    fn minerva_round_one_algorithm_statistics_replay_passes_fixture() {
        let package = synthetic_minerva_round_one_package();
        let transcript = replay_audit_algorithm_statistics(&package.audit_algorithm_runs[0]);

        assert_eq!(transcript.status, AlgorithmReplayStatus::Pass);
        assert_eq!(
            transcript.computed_decision,
            Some(AuditAlgorithmDecision::Pass)
        );
        assert_eq!(transcript.steps.len(), 1);
        assert_eq!(
            transcript.steps[0].statistic,
            RationalValue {
                numerator: 729,
                denominator: 64
            }
        );
        assert_eq!(transcript.steps[0].p_value_ppm, 87_792);
        assert_eq!(transcript.steps[0].step_index, 5);
    }

    #[test]
    fn minerva_multi_round_algorithm_statistics_replays_round_boundaries() {
        let package = synthetic_minerva_multi_round_package();
        let transcript = replay_audit_algorithm_statistics(&package.audit_algorithm_runs[0]);

        assert_eq!(transcript.status, AlgorithmReplayStatus::Pass);
        assert_eq!(
            transcript.computed_decision,
            Some(AuditAlgorithmDecision::Pass)
        );
        assert_eq!(transcript.steps.len(), 2);
        assert_eq!(transcript.steps[0].step_index, 4);
        assert_eq!(transcript.steps[0].p_value_ppm, 131_688);
        assert!(!transcript.steps[0].stop);
        assert_eq!(transcript.steps[1].step_index, 5);
        assert_eq!(transcript.steps[1].p_value_ppm, 87_792);
        assert!(transcript.steps[1].stop);
    }

    #[test]
    fn athena_algorithm_statistics_reports_documented_boundary() {
        let package = synthetic_athena_boundary_package();
        let transcript = replay_audit_algorithm_statistics(&package.audit_algorithm_runs[0]);

        assert_eq!(transcript.status, AlgorithmReplayStatus::Boundary);
        assert_eq!(transcript.decision, AuditAlgorithmDecision::Boundary);
        assert!(transcript
            .boundary
            .as_deref()
            .is_some_and(|boundary| boundary.contains("Athena round risk calculation")));
    }

    #[test]
    fn stratified_hybrid_algorithm_statistics_reports_documented_boundary() {
        let package = synthetic_stratified_hybrid_package();
        let run = package
            .audit_algorithm_runs
            .iter()
            .find(|run| run.method_id == STRATIFIED_HYBRID_RLA_METHOD_ID)
            .expect("stratified run must be present");
        let transcript = replay_audit_algorithm_statistics(run);

        assert_eq!(transcript.status, AlgorithmReplayStatus::Boundary);
        assert_eq!(transcript.decision, AuditAlgorithmDecision::Boundary);
        assert!(transcript
            .boundary
            .as_deref()
            .is_some_and(|boundary| boundary.contains("combined-risk replay")));
    }

    #[test]
    fn raire_algorithm_statistics_reports_documented_boundary() {
        let package = synthetic_raire_boundary_package();
        let transcript = replay_audit_algorithm_statistics(&package.audit_algorithm_runs[0]);

        assert_eq!(transcript.status, AlgorithmReplayStatus::Boundary);
        assert_eq!(transcript.decision, AuditAlgorithmDecision::Boundary);
        assert!(transcript
            .boundary
            .as_deref()
            .is_some_and(|boundary| boundary.contains("RAIRE IRV assertion replay")));
    }

    #[test]
    fn awaire_algorithm_statistics_reports_documented_boundary() {
        let package = synthetic_awaire_boundary_package();
        let transcript = replay_audit_algorithm_statistics(&package.audit_algorithm_runs[0]);

        assert_eq!(transcript.status, AlgorithmReplayStatus::Boundary);
        assert!(transcript
            .boundary
            .as_deref()
            .is_some_and(|boundary| boundary.contains("AWAIRE IRV adaptive replay")));
    }

    #[test]
    fn bayesian_algorithm_statistics_reports_documented_boundary() {
        let package = synthetic_bayesian_tabulation_boundary_package();
        let transcript = replay_audit_algorithm_statistics(&package.audit_algorithm_runs[0]);

        assert_eq!(transcript.status, AlgorithmReplayStatus::Boundary);
        assert_eq!(transcript.decision, AuditAlgorithmDecision::Boundary);
        assert!(transcript
            .boundary
            .as_deref()
            .is_some_and(|boundary| boundary.contains("not risk-limiting replay")));
    }

    #[test]
    fn soba_algorithm_statistics_reports_documented_boundary() {
        let package = synthetic_soba_observable_ballot_boundary_package();
        let transcript = replay_audit_algorithm_statistics(&package.audit_algorithm_runs[0]);

        assert_eq!(transcript.status, AlgorithmReplayStatus::Boundary);
        assert_eq!(transcript.decision, AuditAlgorithmDecision::Boundary);
        assert!(transcript
            .boundary
            .as_deref()
            .is_some_and(|boundary| boundary.contains("observable-ballot linkage")));
    }

    #[test]
    fn alpha_algorithm_statistics_replay_detects_declared_drift() {
        let mut run = AuditAlgorithmRun {
            run_id: "audit-run:alpha-toy".to_string(),
            contest_id: "syn-2024-mayor".to_string(),
            method_id: ALPHA_MARTINGALE_METHOD_ID.to_string(),
            sampling_mode: rcount_core::AuditSamplingMode::WithReplacement,
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
            assertions: vec![rcount_core::AuditAssertion {
                assertion_id: "assertion:cand-a-over-cand-b".to_string(),
                kind: rcount_core::AuditAssertionKind::AssorterMean,
                assorter_id: "toy-assorter-v1".to_string(),
                assorter_upper_bound: RationalValue {
                    numerator: 1,
                    denominator: 1,
                },
                winner_selection_id: Some("cand-a".to_string()),
                loser_selection_id: Some("cand-b".to_string()),
            }],
            sample_steps: (0..4)
                .map(|step_index| rcount_core::AuditSampleStep {
                    step_index,
                    round_index: None,
                    assertion_id: "assertion:cand-a-over-cand-b".to_string(),
                    sample_unit_id: format!("ballot:{step_index}"),
                    assorter_value: RationalValue {
                        numerator: 1,
                        denominator: 1,
                    },
                    bet: Some(RationalValue {
                        numerator: 1,
                        denominator: 1,
                    }),
                    statistic: None,
                    p_value_ppm: None,
                    ranked_choices: Vec::new(),
                    source_refs: Vec::new(),
                })
                .collect(),
            decision: AuditAlgorithmDecision::Pass,
            source_refs: Vec::new(),
        };
        run.sample_steps[3].p_value_ppm = Some(999_999);

        let transcript = replay_audit_algorithm_statistics(&run);
        assert_eq!(transcript.status, AlgorithmReplayStatus::Fail);
        assert!(transcript.steps[3]
            .error
            .as_deref()
            .is_some_and(|error| error.contains("p-value mismatch")));
    }

    #[test]
    fn alpha_algorithm_statistics_reports_boundary_when_bets_are_missing() {
        let run = AuditAlgorithmRun {
            run_id: "audit-run:alpha-boundary".to_string(),
            contest_id: "syn-2024-mayor".to_string(),
            method_id: ALPHA_MARTINGALE_METHOD_ID.to_string(),
            sampling_mode: rcount_core::AuditSamplingMode::WithReplacement,
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
            assertions: vec![rcount_core::AuditAssertion {
                assertion_id: "assertion:cand-a-over-cand-b".to_string(),
                kind: rcount_core::AuditAssertionKind::AssorterMean,
                assorter_id: "toy-assorter-v1".to_string(),
                assorter_upper_bound: RationalValue {
                    numerator: 1,
                    denominator: 1,
                },
                winner_selection_id: None,
                loser_selection_id: None,
            }],
            sample_steps: vec![rcount_core::AuditSampleStep {
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
        };

        let transcript = replay_audit_algorithm_statistics(&run);
        assert_eq!(transcript.status, AlgorithmReplayStatus::Boundary);
        assert!(transcript
            .boundary
            .as_deref()
            .is_some_and(|boundary| boundary.contains("requires bet")));
    }
}
