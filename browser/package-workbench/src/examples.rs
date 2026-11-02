//! Generate complete synthetic packages with RCOUNT's existing native writer.
use rcount_core::*;
use std::path::PathBuf;

fn toy(alpha: bool, drift: bool) -> RcountPackage {
    let mut package = synthetic_minerva_round_one_package();
    let run = &mut package.audit_algorithm_runs[0];
    run.run_id = format!("example:{}", if alpha { "alpha" } else { "bravo" });
    run.method_id = if alpha {
        ALPHA_MARTINGALE_METHOD_ID
    } else {
        BRAVO_BALLOT_POLLING_METHOD_ID
    }
    .to_string();
    run.risk_limit_ppm = Some(if alpha { 250_000 } else { 100_000 });
    run.reported_winner_votes = if alpha { None } else { Some(3) };
    run.reported_loser_votes = if alpha { None } else { Some(1) };
    run.assertions[0].kind = if alpha {
        AuditAssertionKind::AssorterMean
    } else {
        AuditAssertionKind::PluralityWinnerLoser
    };
    run.assertions[0].assorter_upper_bound = RationalValue {
        numerator: 1,
        denominator: 1,
    };
    let mut step = run.sample_steps[0].clone();
    step.round_index = None;
    step.assorter_value = RationalValue {
        numerator: 1,
        denominator: 1,
    };
    step.bet = if alpha {
        Some(RationalValue {
            numerator: 1,
            denominator: 1,
        })
    } else {
        None
    };
    step.statistic = None;
    step.p_value_ppm = None;
    run.sample_steps = (0..if alpha { 4 } else { 6 })
        .map(|i| {
            let mut step = step.clone();
            step.step_index = i;
            step.sample_unit_id = format!("example:ballot:{i}");
            step
        })
        .collect();
    if drift {
        run.sample_steps.last_mut().unwrap().p_value_ppm = Some(999_999);
    }
    run.decision = AuditAlgorithmDecision::Pass;
    package
}

fn main() {
    let root = PathBuf::from(
        std::env::args_os()
            .nth(1)
            .expect("pass a new output directory"),
    );
    assert!(!root.exists(), "output directory must be new");
    let examples = vec![
        ("bravo", toy(false, false)),
        ("bravo-drift", toy(false, true)),
        ("alpha", toy(true, false)),
        ("alpha-drift", toy(true, true)),
        (
            "batch-comparison",
            synthetic_batch_comparison_algorithm_package(),
        ),
        (
            "batch-comparison-drift",
            synthetic_bad_batch_comparison_algorithm_package(),
        ),
        (
            "kaplan-markov-macro",
            synthetic_kaplan_markov_macro_package(),
        ),
        ("minerva-round-one", synthetic_minerva_round_one_package()),
        (
            "minerva-multi-round",
            synthetic_minerva_multi_round_package(),
        ),
        ("athena-boundary", synthetic_athena_boundary_package()),
        (
            "stratified-hybrid-boundary",
            synthetic_stratified_hybrid_package(),
        ),
        ("raire-boundary", synthetic_raire_boundary_package()),
        ("awaire-boundary", synthetic_awaire_boundary_package()),
        (
            "bayesian-boundary",
            synthetic_bayesian_tabulation_boundary_package(),
        ),
        (
            "soba-boundary",
            synthetic_soba_observable_ballot_boundary_package(),
        ),
    ];
    for (name, package) in examples {
        let manifest = rcount_io::synthetic_summary_basic_manifest(&package).unwrap();
        rcount_io::write_package_dir(&root.join(name), &manifest, &package).unwrap();
    }
}
