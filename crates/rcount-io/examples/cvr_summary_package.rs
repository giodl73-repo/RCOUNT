use rcount_core::{synthetic_cvr_summary_package, verify_package};
use rcount_io::{
    default_cvr_summary_docs_dir, synthetic_summary_basic_manifest, write_package_dir,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let package = synthetic_cvr_summary_package();
    verify_package(&package)?;
    let manifest = synthetic_summary_basic_manifest(&package)?;
    let dir = default_cvr_summary_docs_dir();
    write_package_dir(&dir, &manifest, &package)?;
    println!("wrote {}", dir.display());
    Ok(())
}
