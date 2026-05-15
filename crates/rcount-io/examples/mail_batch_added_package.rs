use rcount_core::synthetic_mail_batch_added_package;
use rcount_io::{
    default_mail_batch_added_docs_dir, synthetic_summary_basic_manifest, write_package_dir,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let package = synthetic_mail_batch_added_package();
    let manifest = synthetic_summary_basic_manifest(&package)?;
    let dir = default_mail_batch_added_docs_dir();
    write_package_dir(&dir, &manifest, &package)?;
    println!("wrote {}", dir.display());
    Ok(())
}
