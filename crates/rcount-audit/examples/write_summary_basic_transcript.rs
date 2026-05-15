use rcount_audit::verify_and_write_transcript;
use rcount_io::default_summary_basic_docs_dir;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir = default_summary_basic_docs_dir();
    let transcript = verify_and_write_transcript(&dir)?;
    println!(
        "wrote {} with status {:?}",
        dir.join("transcripts")
            .join("verify-transcript.json")
            .display(),
        transcript.status
    );
    Ok(())
}
