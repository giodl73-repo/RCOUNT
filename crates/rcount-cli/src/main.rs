use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use rcount_audit::{
    replay_audit_algorithm_statistics, verify_package_dir, write_verification_transcript,
    AlgorithmReplayStatus, VerificationStatus,
};
use rcount_core::CountStatus;
use rcount_district::aggregate_package_dir_with_plan_path;
use rcount_io::{
    import_nist_cdf_json, import_ri_2024_rep28_ballot_polling_audit, import_statement_csv,
    read_package_dir, ri_2024_rep28_manifest, synthetic_summary_basic_manifest,
    write_nist_cdf_package_dir, write_ri_2024_rep28_package_dir, write_statement_csv_package_dir,
};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "rcount")]
#[command(about = "RCOUNT election-count package verifier")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Verify(VerifyArgs),
    ReplayAuditAlgorithms(ReplayAuditAlgorithmsArgs),
    AggregateDistricts(AggregateDistrictsArgs),
    ImportStatementCsv(ImportStatementCsvArgs),
    ImportNistCdfJson(ImportNistCdfJsonArgs),
    ImportRi2024Rep28Rla(ImportRi2024Rep28RlaArgs),
}

#[derive(Debug, Parser)]
struct VerifyArgs {
    package_dir: PathBuf,
    #[arg(long)]
    write_transcript: bool,
    #[arg(long)]
    output: Option<PathBuf>,
    #[arg(long, value_enum, default_value = "pretty-json")]
    format: OutputFormat,
}

#[derive(Debug, Parser)]
struct ReplayAuditAlgorithmsArgs {
    package_dir: PathBuf,
    #[arg(long)]
    output: Option<PathBuf>,
    #[arg(long, value_enum, default_value = "pretty-json")]
    format: OutputFormat,
}

#[derive(Debug, Parser)]
struct AggregateDistrictsArgs {
    package_dir: PathBuf,
    #[arg(long)]
    plan: PathBuf,
    #[arg(long)]
    context: Option<PathBuf>,
    #[arg(long)]
    crosswalk: Option<PathBuf>,
    #[arg(long, default_value = "syn-2024-mayor")]
    contest_id: String,
    #[arg(long, default_value = "canvassed")]
    status: String,
    #[arg(long)]
    output: Option<PathBuf>,
    #[arg(long, value_enum, default_value = "pretty-json")]
    format: OutputFormat,
}

#[derive(Debug, Parser)]
struct ImportStatementCsvArgs {
    csv: PathBuf,
    output_dir: PathBuf,
}

#[derive(Debug, Parser)]
struct ImportNistCdfJsonArgs {
    json: PathBuf,
    output_dir: PathBuf,
}

#[derive(Debug, Parser)]
struct ImportRi2024Rep28RlaArgs {
    audit_report_csv: PathBuf,
    ballot_manifest_csv: PathBuf,
    ballot_retrieval_csv: PathBuf,
    output_dir: PathBuf,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum OutputFormat {
    Json,
    PrettyJson,
}

fn main() {
    match run() {
        Ok(code) => std::process::exit(code),
        Err(err) => {
            eprintln!("{err:#}");
            std::process::exit(2);
        }
    }
}

fn run() -> Result<i32> {
    match Cli::parse().command {
        Commands::Verify(args) => run_verify(args),
        Commands::ReplayAuditAlgorithms(args) => run_replay_audit_algorithms(args),
        Commands::AggregateDistricts(args) => run_aggregate_districts(args),
        Commands::ImportStatementCsv(args) => run_import_statement_csv(args),
        Commands::ImportNistCdfJson(args) => run_import_nist_cdf_json(args),
        Commands::ImportRi2024Rep28Rla(args) => run_import_ri_2024_rep28_rla(args),
    }
}

fn run_verify(args: VerifyArgs) -> Result<i32> {
    let transcript = verify_package_dir(&args.package_dir);
    if args.write_transcript {
        write_verification_transcript(&args.package_dir, &transcript).with_context(|| {
            format!(
                "writing transcript under {}",
                args.package_dir.join("transcripts").display()
            )
        })?;
    }

    let output = match args.format {
        OutputFormat::Json => serde_json::to_string(&transcript)?,
        OutputFormat::PrettyJson => serde_json::to_string_pretty(&transcript)?,
    };
    if let Some(path) = &args.output {
        std::fs::write(path, output).with_context(|| format!("writing {}", path.display()))?;
    } else {
        println!("{output}");
    }

    match transcript.status {
        VerificationStatus::Pass => Ok(0),
        VerificationStatus::Fail => Ok(1),
    }
}

fn run_replay_audit_algorithms(args: ReplayAuditAlgorithmsArgs) -> Result<i32> {
    let (_manifest, package) = read_package_dir(&args.package_dir)
        .with_context(|| format!("reading RCOUNT package {}", args.package_dir.display()))?;
    let transcripts = package
        .audit_algorithm_runs
        .iter()
        .map(replay_audit_algorithm_statistics)
        .collect::<Vec<_>>();
    let output = match args.format {
        OutputFormat::Json => serde_json::to_string(&transcripts)?,
        OutputFormat::PrettyJson => serde_json::to_string_pretty(&transcripts)?,
    };
    if let Some(path) = &args.output {
        std::fs::write(path, output).with_context(|| format!("writing {}", path.display()))?;
    } else {
        println!("{output}");
    }

    if transcripts
        .iter()
        .any(|transcript| transcript.status == AlgorithmReplayStatus::Fail)
    {
        Ok(1)
    } else {
        Ok(0)
    }
}

fn run_aggregate_districts(args: AggregateDistrictsArgs) -> Result<i32> {
    let status = parse_count_status(&args.status)?;
    let transcript = aggregate_package_dir_with_plan_path(
        &args.package_dir,
        &args.plan,
        args.context.as_deref(),
        args.crosswalk.as_deref(),
        &args.contest_id,
        status,
    )?;
    let output = match args.format {
        OutputFormat::Json => serde_json::to_string(&transcript)?,
        OutputFormat::PrettyJson => serde_json::to_string_pretty(&transcript)?,
    };
    if let Some(path) = &args.output {
        std::fs::write(path, output).with_context(|| format!("writing {}", path.display()))?;
    } else {
        println!("{output}");
    }
    Ok(0)
}

fn run_import_statement_csv(args: ImportStatementCsvArgs) -> Result<i32> {
    let package = import_statement_csv(&args.csv)
        .with_context(|| format!("importing statement CSV {}", args.csv.display()))?;
    let manifest = synthetic_summary_basic_manifest(&package)?;
    write_statement_csv_package_dir(&args.output_dir, &args.csv, &manifest, &package)
        .with_context(|| format!("writing RCOUNT package {}", args.output_dir.display()))?;
    Ok(0)
}

fn run_import_nist_cdf_json(args: ImportNistCdfJsonArgs) -> Result<i32> {
    let package = import_nist_cdf_json(&args.json)
        .with_context(|| format!("importing NIST CDF JSON {}", args.json.display()))?;
    let manifest = synthetic_summary_basic_manifest(&package)?;
    write_nist_cdf_package_dir(&args.output_dir, &args.json, &manifest, &package)
        .with_context(|| format!("writing RCOUNT package {}", args.output_dir.display()))?;
    Ok(0)
}

fn run_import_ri_2024_rep28_rla(args: ImportRi2024Rep28RlaArgs) -> Result<i32> {
    let package = import_ri_2024_rep28_ballot_polling_audit(
        &args.audit_report_csv,
        &args.ballot_manifest_csv,
        &args.ballot_retrieval_csv,
    )
    .with_context(|| {
        format!(
            "importing Rhode Island 2024 Rep 28 RLA {}, {}, {}",
            args.audit_report_csv.display(),
            args.ballot_manifest_csv.display(),
            args.ballot_retrieval_csv.display()
        )
    })?;
    let manifest = ri_2024_rep28_manifest(&package)?;
    write_ri_2024_rep28_package_dir(
        &args.output_dir,
        &args.audit_report_csv,
        &args.ballot_manifest_csv,
        &args.ballot_retrieval_csv,
        &manifest,
        &package,
    )
    .with_context(|| format!("writing RCOUNT package {}", args.output_dir.display()))?;
    Ok(0)
}

fn parse_count_status(value: &str) -> Result<CountStatus> {
    match value {
        "unofficial" => Ok(CountStatus::Unofficial),
        "canvassed" => Ok(CountStatus::Canvassed),
        "recounted" => Ok(CountStatus::Recounted),
        "amended" => Ok(CountStatus::Amended),
        "certified" => Ok(CountStatus::Certified),
        "withdrawn" => Ok(CountStatus::Withdrawn),
        "superseded" => Ok(CountStatus::Superseded),
        other => anyhow::bail!("unsupported count status: {other}"),
    }
}
