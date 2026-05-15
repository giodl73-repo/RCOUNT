use rcount_core::synthetic_summary_basic_package;
use rcount_io::{
    default_summary_basic_docs_dir, synthetic_summary_basic_manifest, verify_summary_basic_dir,
    write_package_dir,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let package = synthetic_summary_basic_package();
    let manifest = synthetic_summary_basic_manifest(&package)?;
    let dir = default_summary_basic_docs_dir();
    write_package_dir(&dir, &manifest, &package)?;
    verify_summary_basic_dir(&dir)?;
    println!("wrote {}", dir.display());
    Ok(())
}
