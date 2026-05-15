use rcount_core::synthetic_precinct_split_lineage_package;
use rcount_io::{
    default_precinct_split_lineage_docs_dir, synthetic_summary_basic_manifest, write_package_dir,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let package = synthetic_precinct_split_lineage_package();
    let manifest = synthetic_summary_basic_manifest(&package)?;
    let dir = default_precinct_split_lineage_docs_dir();
    write_package_dir(&dir, &manifest, &package)?;
    println!("wrote {}", dir.display());
    Ok(())
}
