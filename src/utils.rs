use std::path::Path;
use cargo_metadata::Metadata;

use crate::error::Error;

pub fn get_project_metadata<P: AsRef<Path>>(manifest_path: P) -> Result<Metadata, Error> {
    let mut metadata_cmd = cargo_metadata::MetadataCommand::new();
    metadata_cmd.manifest_path(manifest_path);

    let project_metadata = metadata_cmd.exec()?;

    Ok(project_metadata)
}