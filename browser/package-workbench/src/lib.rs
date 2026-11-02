use rcount_audit::{replay_audit_algorithm_statistics, verify_package_with_reader};
use rcount_io::{read_package_with_reader, PackageReader};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::io::{self, BufRead, Cursor};
use std::path::{Path, PathBuf};
use wasm_bindgen::prelude::*;

pub const MAX_REQUEST_BYTES: usize = 4_194_304;
const MAX_TOTAL_BYTES: usize = 524_288;
const MAX_FILE_BYTES: usize = 131_072;
const MAX_REPORT_BYTES: usize = 2_097_152;
const SCOPE: &str = "Consistency and statistical replay of supplied package declarations. Sources, authority and status labels are unverified. Passing checks do not certify an election, approve canvassing or district plans, or authenticate evidence. Boundary methods are not statistically replayed.";
const DATA_FILES: &[&str] = &[
    "manifest.json",
    "sources/source-index.json",
    "normalized/contests.ndjson",
    "normalized/reporting-units.ndjson",
    "normalized/batches.ndjson",
    "normalized/lineage.ndjson",
    "normalized/rhist-refs.ndjson",
    "normalized/rctx-refs.ndjson",
    "proofs/inclusion-proofs.ndjson",
    "normalized/cvr.ndjson",
    "audits/algorithm-runs.ndjson",
    "audits/rla.ndjson",
    "audits/manual.ndjson",
    "audits/batch-comparison.ndjson",
    "normalized/summaries.ndjson",
    "status/events.ndjson",
];

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct InputFile {
    path: String,
    bytes: Vec<u8>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    files: Vec<InputFile>,
}

struct Files(BTreeMap<PathBuf, Vec<u8>>);

impl PackageReader for Files {
    fn exists(&self, path: &Path) -> bool {
        self.0.contains_key(path)
    }
    fn open(&self, path: &Path) -> io::Result<Box<dyn BufRead + '_>> {
        let bytes = self.0.get(path).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!(
                    "package file is missing: {}",
                    path.display().to_string().replace('\\', "/")
                ),
            )
        })?;
        Ok(Box::new(Cursor::new(bytes)))
    }
}

fn diagnostic(kind: &str, message: impl ToString) -> Value {
    json!({"kind":kind,"message":message.to_string(),"scope":SCOPE})
}

pub fn refusal(message: &str) -> String {
    diagnostic("refused", message).to_string()
}

fn canonical_path(path: &str) -> bool {
    !path.is_empty()
        && path.len() <= 256
        && path.split('/').all(|part| {
            !part.is_empty()
                && part != "."
                && part != ".."
                && !part.ends_with('.')
                && part
                    .bytes()
                    .all(|c| c.is_ascii_alphanumeric() || b"._-".contains(&c))
                && {
                    let stem = part.split('.').next().unwrap().to_ascii_uppercase();
                    !matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
                        && !(stem.len() == 4
                            && (stem.starts_with("COM") || stem.starts_with("LPT"))
                            && matches!(stem.as_bytes()[3], b'1'..=b'9'))
                }
        })
}

// Admission is distinct from native validity. Preserve native diagnostics for
// malformed records; only refuse inputs whose structure or work exceeds bounds.
fn bound_value(value: &Value, nodes: &mut usize) -> Result<(), Value> {
    let mut stack = vec![value];
    while let Some(value) = stack.pop() {
        *nodes += 1;
        if *nodes > 65_536 {
            return Err(diagnostic("refused", "JSON exceeds 65536 nodes"));
        }
        match value {
            Value::Array(items) => {
                if items.len() > 512 {
                    return Err(diagnostic("refused", "Array exceeds 512 entries"));
                }
                stack.extend(items);
            }
            Value::Object(items) => {
                if items.keys().any(|key| key.len() > 256) {
                    return Err(diagnostic("refused", "JSON key exceeds 256 bytes"));
                }
                stack.extend(items.values());
            }
            Value::String(text) if text.len() > 16_384 => {
                return Err(diagnostic("refused", "String exceeds 16 KiB"))
            }
            Value::Number(number)
                if number
                    .as_f64()
                    .is_none_or(|n| n.abs() > 1_000_000_000_000.0) =>
            {
                return Err(diagnostic(
                    "refused",
                    "Numeric magnitude exceeds 1000000000000",
                ))
            }
            _ => {}
        }
    }
    Ok(())
}

fn inspect_files(request: Request) -> Result<Files, Value> {
    if request.files.is_empty() || request.files.len() > 64 {
        return Err(diagnostic(
            "refused",
            "Choose between 1 and 64 package files",
        ));
    }
    let mut total = 0;
    let mut nodes = 0;
    let mut records = 0;
    let mut files = BTreeMap::new();
    for file in request.files {
        if !canonical_path(&file.path) {
            return Err(diagnostic(
                "refused",
                format!("Noncanonical package path: {}", file.path),
            ));
        }
        total += file.bytes.len();
        if file.bytes.len() > MAX_FILE_BYTES || total > MAX_TOTAL_BYTES {
            return Err(diagnostic(
                "refused",
                "Limits: 128 KiB per file, 512 KiB total",
            ));
        }
        if files
            .insert(PathBuf::from(&file.path), file.bytes.clone())
            .is_some()
        {
            return Err(diagnostic("refused", "Duplicate package path"));
        }
        if DATA_FILES.contains(&file.path.as_str()) {
            // Native parsing owns malformed JSON/UTF-8 diagnostics. Bound any
            // successfully parsed records before invoking its verifier.
            let lines: Vec<&[u8]> = if file.path.ends_with(".ndjson") {
                file.bytes.split(|byte| *byte == b'\n').collect()
            } else {
                vec![&file.bytes]
            };
            for line in lines {
                if line.iter().all(u8::is_ascii_whitespace) {
                    continue;
                }
                records += 1;
                if records > 2048 {
                    return Err(diagnostic("refused", "Package exceeds 2048 records"));
                }
                if let Ok(value) = serde_json::from_slice::<Value>(line) {
                    bound_value(&value, &mut nodes)?;
                    if file.path == "sources/source-index.json" {
                        if let Some(sources) = value.get("sources").and_then(Value::as_array) {
                            for source in sources {
                                if let Some(path) = source.get("path").and_then(Value::as_str) {
                                    if !canonical_path(path) || !path.starts_with("sources/") {
                                        return Err(diagnostic(
                                            "refused",
                                            "Source index requires canonical paths under sources/",
                                        ));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(Files(files))
}

fn report(input: &str) -> Result<Value, Value> {
    let request: Request = serde_json::from_str(input).map_err(|e| diagnostic("error", e))?;
    let files = inspect_files(request)?;
    let root = Path::new("");
    // Loading only hashes/parses records. Bound numeric loop drivers and nested
    // replay work before calling the native package verifier or replay routines.
    if let Ok((_, package)) = read_package_with_reader(&files, root) {
        if package.rla_audits.iter().any(|a| a.sample_size > 256)
            || package
                .rla_audits
                .iter()
                .map(|a| u64::from(a.sample_size))
                .sum::<u64>()
                > 1024
        {
            return Err(diagnostic(
                "refused",
                "RLA limits: 256 draws per audit, 1024 total",
            ));
        }
        let total_steps: usize = package
            .audit_algorithm_runs
            .iter()
            .map(|r| r.sample_steps.len())
            .sum();
        if package.audit_algorithm_runs.len() > 16
            || total_steps > 256
            || package.audit_algorithm_runs.iter().any(|r| {
                r.method_id == rcount_core::MINERVA_BALLOT_POLLING_METHOD_ID
                    && r.sample_steps.len() > 64
            })
        {
            return Err(diagnostic(
                "refused",
                "Replay limits: 16 runs, 256 total steps, 64 Minerva steps",
            ));
        }
    }
    let verification = verify_package_with_reader(&files, root);
    let replay = match read_package_with_reader(&files, root) {
        Ok((_, package)) => {
            json!({"kind":"loaded","transcripts":package.audit_algorithm_runs.iter().map(replay_audit_algorithm_statistics).collect::<Vec<_>>() })
        }
        Err(error) => json!({"kind":"unavailable","message":error.to_string()}),
    };
    Ok(json!({"kind":"report","scope":SCOPE,"verification":verification,"replay":replay}))
}

#[wasm_bindgen]
pub fn engine_info() -> String {
    json!({"scope":SCOPE,"request_bytes":MAX_REQUEST_BYTES,"file_bytes":MAX_FILE_BYTES,
        "total_bytes":MAX_TOTAL_BYTES,"files":64,"records":2048,"report_bytes":MAX_REPORT_BYTES,
        "rla_draws_per_audit":256,"rla_draws_total":1024,"replay_runs":16,"replay_steps":256,"minerva_steps":64}).to_string()
}

#[wasm_bindgen]
pub fn evaluate(input: &str) -> String {
    if input.len() > MAX_REQUEST_BYTES {
        return refusal("Request exceeds 4 MiB");
    }
    let output = report(input).unwrap_or_else(|error| error).to_string();
    if output.len() > MAX_REPORT_BYTES {
        refusal("Report exceeds 2 MiB")
    } else {
        output
    }
}
