use rcount_core::{replay_rla_sample, synthetic_rla_replay_package};

#[test]
fn seeded_sampler_retains_the_original_64_bit_golden_draws() {
    // Frozen original fixture values, rather than values derived by this test.
    // The wasm workbench also verifies the original disk fixture with these draws.
    let package = synthetic_rla_replay_package();
    let draws = replay_rla_sample(&package, &package.rla_audits[0]).unwrap();
    let expected = [
        "cvr:P-001:022",
        "cvr:P-001:074",
        "cvr:P-002:030",
        "cvr:P-001:056",
        "cvr:P-002:059",
        "cvr:P-002:053",
        "cvr:P-002:046",
        "cvr:P-001:056",
        "cvr:P-002:001",
        "cvr:P-001:042",
        "cvr:P-002:024",
        "cvr:P-002:052",
    ];
    assert_eq!(
        draws
            .iter()
            .map(|draw| draw.cvr_id.as_str())
            .collect::<Vec<_>>(),
        expected
    );
    assert_eq!(
        draws.iter().map(|draw| draw.draw_index).collect::<Vec<_>>(),
        (0..12).collect::<Vec<_>>()
    );
}
