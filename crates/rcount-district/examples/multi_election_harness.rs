use rcount_district::{synthetic_multi_election_harness, verify_synthetic_multi_election_harness};
use rcount_io::{synthetic_summary_basic_manifest, write_package_dir};
use rplan_io::write_rplan_string;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let base_dir = std::path::PathBuf::from("docs")
        .join("examples")
        .join("rcount-golden-packages")
        .join("multi-election-harness");
    std::fs::create_dir_all(&base_dir)?;

    let harness = synthetic_multi_election_harness()?;
    for cycle in &harness.cycles {
        let cycle_dir = base_dir.join(&cycle.cycle_id);
        let package_dir = cycle_dir.join("package");
        let manifest = synthetic_summary_basic_manifest(&cycle.package)?;
        write_package_dir(&package_dir, &manifest, &cycle.package)?;
        std::fs::write(
            cycle_dir.join("plan.rplan.json"),
            write_rplan_string(&cycle.plan)?,
        )?;
    }

    let transcript = verify_synthetic_multi_election_harness(&harness)?;
    std::fs::write(
        base_dir.join("multi-election-transcript.json"),
        serde_json::to_vec_pretty(&transcript)?,
    )?;
    println!("wrote {}", base_dir.display());
    Ok(())
}
