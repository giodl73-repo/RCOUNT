use serde_json::Value;

fn normalized(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[test]
fn verification_transcript_does_not_become_certification_claim() {
    // Checks RCOUNT-PF-01: package verification is not public/legal authority.
    let readme = include_str!("../../../README.md");
    let contract = include_str!("../../../docs/family-contract.md");
    let pitfalls = include_str!("../../../.pitfall/rcount-pitfalls.md");
    let boundaries: Value =
        serde_json::from_str(include_str!("../../../docs/authority-boundaries.v1.json"))
            .expect("authority boundaries must remain valid JSON");

    assert!(normalized(readme).contains("verification packages do not inherit BISECT"));
    assert!(contract.contains("RCOUNT does not certify an election"));
    assert!(normalized(pitfalls).contains("legal election certification"));

    let does_not_prove = boundaries["verification_transcript_boundary"]["does_not_prove"]
        .as_array()
        .expect("verification boundary should list forbidden certification claims");
    for forbidden in [
        "legal_election_certification",
        "canvassing_approval",
        "district_plan_approval",
        "election_night_reporting",
        "public_result_authority",
    ] {
        assert!(does_not_prove.iter().any(|claim| claim == forbidden));
    }
}

#[test]
fn thin_bridge_does_not_copy_sibling_ownership() {
    // Checks RCOUNT-PF-04: RPLAN/RCTX/RHIST bridges stay hash-based and owned.
    let spec = include_str!("../../../docs/specs/rcount-foundation.md");
    let contract = include_str!("../../../docs/family-contract.md");
    let plan_fixture = include_str!(
        "../../../docs/examples/rcount-golden-packages/district-aggregation-rplan/plan.rplan.json"
    );
    let boundaries: Value =
        serde_json::from_str(include_str!("../../../docs/authority-boundaries.v1.json"))
            .expect("authority boundaries must remain valid JSON");

    assert!(spec.contains("RCOUNT owns count evidence and audit replay primitives"));
    assert!(spec.contains("It does not own RPLAN"));
    assert!(contract.contains("dependency migration must not move BISECT-owned policy into RCOUNT"));
    assert!(
        plan_fixture.contains("RPLAN records assignments only; RHIST owns cross-cycle lineage.")
    );

    let bridge = &boundaries["bridge_boundary"];
    assert_eq!(bridge["rplan"], "assignment_and_plan_hash_only");
    assert_eq!(bridge["rctx"], "context_and_crosswalk_hash_only");
    assert_eq!(bridge["rhist"], "lineage_reference_only");
    assert_eq!(bridge["bisect"], "consumer_application_policy_only");
    assert_eq!(
        bridge["rcount_role"],
        "count_package_verification_and_aggregation_transcript"
    );
    let forbidden = bridge["forbidden_inferences"]
        .as_array()
        .expect("bridge boundary should list forbidden ownership inferences");
    assert!(forbidden
        .iter()
        .any(|claim| claim == "green_bridge_test_replaces_bisect_migration_evidence"));
}
