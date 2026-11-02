use rcount_package_workbench::{evaluate, MAX_REQUEST_BYTES};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

fn example(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/examples/rcount-golden-packages")
        .join(name)
}

fn request(name: &str) -> Value {
    let root = example(name);
    fn visit(root: &Path, dir: &Path, files: &mut Vec<Value>) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                visit(root, &path, files);
            } else {
                files.push(json!({"path":path.strip_prefix(root).unwrap().to_string_lossy().replace('\\',"/"),"bytes":std::fs::read(path).unwrap()}));
            }
        }
    }
    let mut files = Vec::new();
    visit(&root, &root, &mut files);
    json!({"files":files})
}

fn run(request: Value) -> Value {
    serde_json::from_str(&evaluate(&request.to_string())).unwrap()
}

fn replace(request: &mut Value, path: &str, bytes: Vec<u8>) {
    request["files"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|file| file["path"] == path)
        .unwrap()["bytes"] = json!(bytes);
}

#[test]
fn complete_synthetic_package_verifies() {
    let report = run(request("summary-basic"));
    assert_eq!(report["kind"], "report");
    assert_eq!(report["verification"]["status"], "pass");
}

#[test]
fn raw_binary_source_is_not_decoded_or_normalized() {
    let mut input = request("summary-basic");
    replace(
        &mut input,
        "sources/synthetic-summary-export.json",
        vec![0xff, 0x00, 0x0d, 0x0a],
    );
    let report = run(input);
    assert_eq!(report["kind"], "report");
    assert_eq!(report["verification"]["status"], "fail");
    assert!(report["verification"]["checks"]
        .as_array()
        .unwrap()
        .iter()
        .any(|c| c["error"]
            .as_str()
            .is_some_and(|s| s.contains("source hash mismatch"))));
}

#[test]
fn path_aliases_absolute_paths_and_reserved_names_are_refused() {
    for path in [
        "../manifest.json",
        "/manifest.json",
        "sources\\file.json",
        "C:/manifest.json",
        "sources/./file",
        "sources//file",
        "sources/NUL.txt",
        "sources/COM1",
        "sources/name.",
        "sources/name ",
    ] {
        assert_eq!(
            run(json!({"files":[{"path":path,"bytes":[]}]}))["kind"],
            "refused",
            "{path}"
        );
    }
}

#[test]
fn duplicate_paths_are_refused() {
    assert_eq!(
        run(
            json!({"files":[{"path":"manifest.json","bytes":[]},{"path":"manifest.json","bytes":[]}]})
        )["kind"],
        "refused"
    );
}

#[test]
fn file_count_and_file_size_limits_are_enforced() {
    assert_eq!(run(json!({"files":[]}))["kind"], "refused");
    let files = (0..65)
        .map(|i| json!({"path":format!("sources/{i}.bin"),"bytes":[]}))
        .collect::<Vec<_>>();
    assert_eq!(run(json!({"files":files}))["kind"], "refused");
    assert_eq!(
        run(json!({"files":[{"path":"sources/file.bin","bytes":vec![0_u8;131_073]}]}))["kind"],
        "refused"
    );
}

#[test]
fn total_source_bytes_are_bounded() {
    let files = (0..5)
        .map(|i| json!({"path":format!("sources/{i}.bin"),"bytes":vec![0_u8;110_000]}))
        .collect::<Vec<_>>();
    assert_eq!(run(json!({"files":files}))["kind"], "refused");
}

#[test]
fn oversized_transport_and_unknown_fields_are_rejected() {
    assert_eq!(
        serde_json::from_str::<Value>(&evaluate(&" ".repeat(MAX_REQUEST_BYTES + 1))).unwrap()
            ["kind"],
        "refused"
    );
    assert_eq!(run(json!({"files":[],"unexpected":true}))["kind"], "error");
}

#[test]
fn source_index_traversal_is_refused_before_native_verification() {
    let mut input = request("summary-basic");
    replace(
        &mut input,
        "sources/source-index.json",
        serde_json::to_vec(&json!({"sources":[{"path":"sources/../manifest.json"}]})).unwrap(),
    );
    assert_eq!(run(input)["kind"], "refused");
}

#[test]
fn numeric_magnitude_and_array_bounds_are_enforced() {
    for value in [json!({"x":1_000_000_000_001_u64}), json!({"x":vec![0;513]})] {
        assert_eq!(
            run(
                json!({"files":[{"path":"manifest.json","bytes":serde_json::to_vec(&value).unwrap()}]})
            )["kind"],
            "refused"
        );
    }
}

#[test]
fn missing_required_file_preserves_native_verification_and_replay_failure() {
    let report = run(json!({"files":[{"path":"sources/unused.bin","bytes":[]}]}));
    assert_eq!(
        report["verification"]["checks"][0]["equation_id"],
        "package_read"
    );
    assert_eq!(report["replay"]["kind"], "unavailable");
}

#[test]
fn rla_loop_driver_is_bounded_after_hash_valid_loading() {
    let mut input = request("rla-replay");
    let (mut manifest, mut package) = rcount_io::read_package_dir(&example("rla-replay")).unwrap();
    package.rla_audits[0].sample_size = u32::MAX;
    manifest.content_hash = rcount_core::package_content_hash(&package).unwrap();
    replace(
        &mut input,
        "manifest.json",
        serde_json::to_vec(&manifest).unwrap(),
    );
    replace(
        &mut input,
        "audits/rla.ndjson",
        serde_json::to_vec(&package.rla_audits[0]).unwrap(),
    );
    let report = run(input);
    assert_eq!(report["kind"], "refused");
    assert!(report["message"].as_str().unwrap().contains("RLA limits"));
}

#[test]
fn boundary_replay_does_not_become_pass() {
    let mut input = request("summary-basic");
    let (mut manifest, mut package) =
        rcount_io::read_package_dir(&example("summary-basic")).unwrap();
    package.audit_algorithm_runs =
        rcount_core::synthetic_athena_boundary_package().audit_algorithm_runs;
    manifest.content_hash = rcount_core::package_content_hash(&package).unwrap();
    replace(
        &mut input,
        "manifest.json",
        serde_json::to_vec(&manifest).unwrap(),
    );
    input["files"].as_array_mut().unwrap().push(json!({"path":"audits/algorithm-runs.ndjson","bytes":serde_json::to_vec(&package.audit_algorithm_runs[0]).unwrap()}));
    let report = run(input);
    assert_eq!(report["kind"], "report");
    assert_eq!(report["replay"]["transcripts"][0]["status"], "boundary");
}
