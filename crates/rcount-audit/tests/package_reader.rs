use rcount_audit::{verify_package_dir, verify_package_with_reader, VerificationStatus};
use rcount_io::{read_package_with_reader, PackageReader};
use std::collections::BTreeMap;
use std::io::{self, BufRead, Cursor};
use std::path::{Path, PathBuf};

struct MemoryReader(BTreeMap<PathBuf, Vec<u8>>);

impl PackageReader for MemoryReader {
    fn exists(&self, path: &Path) -> bool {
        self.0.contains_key(path)
    }

    fn open(&self, path: &Path) -> io::Result<Box<dyn BufRead + '_>> {
        let bytes = self.0.get(path).ok_or_else(|| {
            io::Error::new(io::ErrorKind::NotFound, "memory package file is missing")
        })?;
        Ok(Box::new(Cursor::new(bytes)))
    }
}

fn load(dir: &Path) -> MemoryReader {
    fn visit(root: &Path, dir: &Path, files: &mut BTreeMap<PathBuf, Vec<u8>>) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                visit(root, &path, files);
            } else {
                files.insert(
                    path.strip_prefix(root).unwrap().into(),
                    std::fs::read(path).unwrap(),
                );
            }
        }
    }
    let mut files = BTreeMap::new();
    visit(dir, dir, &mut files);
    MemoryReader(files)
}

fn examples() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/examples/rcount-golden-packages")
}

#[test]
fn all_golden_packages_have_identical_filesystem_and_memory_transcripts() {
    fn visit(dir: &Path, count: &mut usize) {
        if dir.join("manifest.json").exists() {
            let memory = load(dir);
            assert_eq!(
                verify_package_dir(dir),
                verify_package_with_reader(&memory, Path::new("")),
                "{}",
                dir.display()
            );
            *count += 1;
        }
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                visit(&path, count);
            }
        }
    }
    let mut count = 0;
    visit(&examples(), &mut count);
    assert_eq!(count, 42);
}

#[test]
fn ndjson_line_endings_and_blank_lines_preserve_package_semantics() {
    let dir = examples().join("summary-basic");
    let mut memory = load(&dir);
    for (path, bytes) in &mut memory.0 {
        if path
            .extension()
            .is_some_and(|extension| extension == "ndjson")
        {
            let text = String::from_utf8(bytes.clone()).unwrap();
            *bytes =
                format!("\r\n  \r\n{}\r\n", text.trim_end().replace('\n', "\r\n")).into_bytes();
        }
    }
    assert_eq!(
        verify_package_dir(&dir),
        verify_package_with_reader(&memory, Path::new(""))
    );
}

#[test]
fn source_bytes_are_hashed_without_newline_normalization() {
    let mut memory = load(&examples().join("summary-basic"));
    memory
        .0
        .get_mut(Path::new("sources/synthetic-summary-export.json"))
        .unwrap()
        .push(b'\n');
    let transcript = verify_package_with_reader(&memory, Path::new(""));
    assert_eq!(transcript.status, VerificationStatus::Fail);
    assert!(transcript
        .checks
        .iter()
        .any(|check| check.equation_id == "source_hash_match"
            && check
                .error
                .as_deref()
                .is_some_and(|error| error.contains("source hash mismatch"))));
}

#[test]
fn missing_source_uses_the_existing_source_gate() {
    let mut memory = load(&examples().join("summary-basic"));
    memory
        .0
        .remove(Path::new("sources/synthetic-summary-export.json"));
    let transcript = verify_package_with_reader(&memory, Path::new(""));
    assert_eq!(transcript.status, VerificationStatus::Fail);
    assert!(transcript.checks.iter().any(|check| check.error.as_deref()
        == Some("source file is missing: sources/synthetic-summary-export.json")));
}

#[test]
fn missing_required_file_is_package_read_failure() {
    let mut memory = load(&examples().join("summary-basic"));
    memory.0.remove(Path::new("normalized/contests.ndjson"));
    let transcript = verify_package_with_reader(&memory, Path::new(""));
    assert_eq!(transcript.status, VerificationStatus::Fail);
    assert_eq!(transcript.checks.len(), 1);
    assert_eq!(transcript.checks[0].equation_id, "package_read");
}

#[test]
fn absent_optional_files_are_empty_collections() {
    let memory = load(&examples().join("summary-basic"));
    assert!(!memory.exists(Path::new("audits/algorithm-runs.ndjson")));
    let (_, package) = read_package_with_reader(&memory, Path::new("")).unwrap();
    assert!(package.audit_algorithm_runs.is_empty());
    assert_eq!(
        verify_package_with_reader(&memory, Path::new("")).status,
        VerificationStatus::Pass
    );
}

#[test]
fn malformed_ndjson_and_invalid_utf8_keep_json_and_io_errors_distinct() {
    let mut memory = load(&examples().join("summary-basic"));
    let path = PathBuf::from("normalized/contests.ndjson");
    memory.0.insert(path.clone(), b"{bad json}\n".to_vec());
    let malformed = verify_package_with_reader(&memory, Path::new(""));
    assert_eq!(malformed.status, VerificationStatus::Fail);
    assert!(malformed.checks[0]
        .error
        .as_ref()
        .unwrap()
        .starts_with("json error:"));
    memory.0.insert(path, vec![0xff, b'\n']);
    let invalid = verify_package_with_reader(&memory, Path::new(""));
    assert_eq!(invalid.status, VerificationStatus::Fail);
    assert!(invalid.checks[0]
        .error
        .as_ref()
        .unwrap()
        .starts_with("io error:"));
}
