use rcount_core::{
    synthetic_summary_basic_package_with_base_references, SYN_RCTX_L0_CROSSWALK_HASH,
    SYN_RCTX_L0_PACKAGE_HASH, SYN_RHIST_L2_PACKAGE_HASH,
};
use rcount_district::{aggregate_package_districts, synthetic_summary_basic_rplan_document};
use rcount_io::{synthetic_summary_basic_manifest, write_package_dir};
use rplan_io::{write_rctx_string, write_rplan_string};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let base_dir = std::path::PathBuf::from("docs")
        .join("examples")
        .join("rcount-golden-packages")
        .join("district-aggregation-rplan");
    let package_dir = base_dir.join("package");
    let mut plan_doc = synthetic_summary_basic_rplan_document()?;
    plan_doc.extensions.insert(
        "civic_evidence_base_references".to_string(),
        serde_json::json!({
            "rctx_package_hash": SYN_RCTX_L0_PACKAGE_HASH,
            "rctx_crosswalk_hash": SYN_RCTX_L0_CROSSWALK_HASH,
            "rhist_package_hash": SYN_RHIST_L2_PACKAGE_HASH,
            "claim_boundary": "RPLAN records assignments only; RHIST owns cross-cycle lineage.",
        }),
    );
    let mut context = rplan_core::RplanContext {
        rctx_version: rplan_core::RCTX_VERSION.to_string(),
        context_hash: String::new(),
        units: plan_doc.plan.units.clone(),
        graph: None,
        populations: None,
        subdivisions: None,
        demographics: None,
        geometry: None,
        source_hashes: rplan_core::SourceHashes::default(),
    };
    context.context_hash = context.compute_context_hash()?;
    let rctx_fixture = rctx_core::synthetic_minimal_package_fixture()?;
    let package = synthetic_summary_basic_package_with_base_references();
    let manifest = synthetic_summary_basic_manifest(&package)?;
    write_package_dir(&package_dir, &manifest, &package)?;

    std::fs::create_dir_all(&base_dir)?;
    std::fs::write(
        base_dir.join("plan.rplan.json"),
        write_rplan_string(&plan_doc)?,
    )?;
    std::fs::write(base_dir.join("context.rctx"), write_rctx_string(&context)?)?;
    let crosswalk_text = rctx_fixture
        .crosswalks
        .iter()
        .map(serde_json::to_string)
        .collect::<Result<Vec<_>, _>>()?
        .join("\n");
    std::fs::write(
        base_dir.join("crosswalks.ndjson"),
        format!("{crosswalk_text}\n"),
    )?;

    let transcript = aggregate_package_districts(
        &package,
        &plan_doc.plan,
        Some(&context),
        None,
        "syn-2024-mayor",
        rcount_core::CountStatus::Canvassed,
    )?;
    std::fs::write(
        base_dir.join("district-aggregation-transcript.json"),
        serde_json::to_vec_pretty(&transcript)?,
    )?;
    println!("wrote {}", base_dir.display());
    Ok(())
}
