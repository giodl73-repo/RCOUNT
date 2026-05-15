use rcount_district::{
    synthetic_bad_lineage_multi_election_harness, synthetic_stale_plan_multi_election_harness,
    verify_synthetic_multi_election_harness, SyntheticMultiElectionHarness,
};
use rcount_io::{synthetic_summary_basic_manifest, write_package_dir};
use rplan_io::write_rplan_string;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let base_dir = std::path::PathBuf::from("docs")
        .join("examples")
        .join("rcount-golden-packages")
        .join("multi-election-harness-negatives");
    std::fs::create_dir_all(&base_dir)?;

    write_harness_case(
        &base_dir.join("bad-lineage"),
        &synthetic_bad_lineage_multi_election_harness()?,
    )?;
    write_harness_case(
        &base_dir.join("stale-plan"),
        &synthetic_stale_plan_multi_election_harness()?,
    )?;
    write_tampered_source_case(&base_dir.join("tampered-2028-source"))?;

    println!("wrote {}", base_dir.display());
    Ok(())
}

fn write_harness_case(
    case_dir: &std::path::Path,
    harness: &SyntheticMultiElectionHarness,
) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::create_dir_all(case_dir)?;
    for cycle in &harness.cycles {
        let cycle_dir = case_dir.join(&cycle.cycle_id);
        let package_dir = cycle_dir.join("package");
        let manifest = synthetic_summary_basic_manifest(&cycle.package)?;
        write_package_dir(&package_dir, &manifest, &cycle.package)?;
        std::fs::write(
            cycle_dir.join("plan.rplan.json"),
            write_rplan_string(&cycle.plan)?,
        )?;
    }

    let failure = verify_synthetic_multi_election_harness(harness)
        .err()
        .map(|err| err.to_string())
        .unwrap_or_else(|| "expected synthetic negative harness to fail".to_string());
    std::fs::write(
        case_dir.join("expected-failure.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "expected_result": "fail",
            "failure": failure
        }))?,
    )?;
    Ok(())
}

fn write_tampered_source_case(
    case_dir: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let harness = rcount_district::synthetic_multi_election_harness()?;
    write_harness_case(case_dir, &harness)?;
    std::fs::write(
        case_dir
            .join("SYN-2028-general")
            .join("package")
            .join("sources")
            .join("synthetic-summary-export.json"),
        br#"{"tampered":true}"#,
    )?;
    std::fs::write(
        case_dir.join("expected-failure.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "expected_result": "fail",
            "failure": "source hash mismatch in SYN-2028-general package"
        }))?,
    )?;
    Ok(())
}
