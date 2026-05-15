use rcount_core::synthetic_canvass_correction_package;
use rcount_io::{
    default_canvass_correction_docs_dir, synthetic_canvass_correction_manifest, write_package_dir,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let package = synthetic_canvass_correction_package();
    let manifest = synthetic_canvass_correction_manifest(&package)?;
    let dir = default_canvass_correction_docs_dir();
    write_package_dir(&dir, &manifest, &package)?;
    println!("wrote {}", dir.display());
    Ok(())
}
