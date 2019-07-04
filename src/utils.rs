use cargo_metadata::Metadata;
use std::path::Path;

use toml::Value;
use crate::error::Error;

pub fn get_project_metadata<P: AsRef<Path>>(manifest_path: P) -> Result<Metadata, Error> {
    let mut metadata_cmd = cargo_metadata::MetadataCommand::new();
    metadata_cmd.no_deps();
    metadata_cmd.manifest_path(manifest_path);

    let project_metadata = metadata_cmd.exec()?;

    Ok(project_metadata)
}

pub fn config_from_file<P: AsRef<Path>>(config_path: P) -> Result<Value, Error> {
    let config_file = std::fs::read_to_string(config_path)?;

    config_file.parse::<Value>().map_err(Error::ParseTomlError)
}

// .or_else(|| {
//     configs
//         .into_iter()
//         .flat_map(|config| config.and_then(|c| c["remote"].as_str().map(String::from)))
//         .next()
// })
// .unwrap_or_else(|| {
//     error!("No remote build server was defined (use config file or --remote flag)");
//     exit(-3);
// });
