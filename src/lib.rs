// use std::fs::PathBuf;
use std::path::{Path, PathBuf};
use std::process::{exit, Command, Stdio};

use toml::Value;

use log::{error, info, warn};

use structopt::StructOpt;

pub mod config;
pub mod error;
pub mod utils;

pub use config::Config;

use error::Error;

pub fn run_config(config: Config) -> Result<(), Error> {
    let Config::Remote {
        remote,
        copy_back,
        manifest_path,
        hidden,
        command,
        options,
    } = config;

    let project_metadata = utils::get_project_metadata(manifest_path)?;

    // for now, assume that there is only one project and find it's root directory
    let project = project_metadata
        .packages
        .first()
        .ok_or(Error::NoProjectFoundError)?;
    
    let project_dir = &project_metadata.workspace_root;

    let project_name = &project.name;

    // TODO: move Opts::Remote fields into own type and implement complete_from_config(&mut self, config: &Value)
    let build_server = remote.unwrap();
    
    let build_path = format!("~/remote-builds/{}/", project_name);

    info!("Transferring sources to build server.");
    // transfer project to build server
    let mut rsync_to = Command::new("rsync");
    rsync_to
        .arg("-a".to_owned())
        .arg("--delete")
        .arg("--info=progress2")
        .arg("--exclude")
        .arg("target");

    if !hidden {
        rsync_to.arg("--exclude").arg(".*");
    }

    rsync_to
        .arg("--rsync-path")
        .arg("mkdir -p remote-builds && rsync")
        .arg(format!("{}/", project_dir.to_string_lossy()))
        .arg(format!("{}:{}", build_server, build_path))
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .stdin(Stdio::inherit());

    println!("Rsync commmand: {:?}", rsync_to);
    
    rsync_to.output()
        .map_err(Error::TransferFilesError)?;

    let build_command = format!(
        "cd {}; $HOME/.cargo/bin/cargo {} {}",
        build_path,
        command,
        options.join(" ")
    );

    info!("Starting build process.");
    Command::new("ssh")
        .arg("-t")
        .arg(&build_server)
        .arg(build_command)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .stdin(Stdio::inherit())
        .output()
        .map_err(Error::RunCargoCommandError)?;

    if copy_back {
        info!("Transferring artifacts back to client.");
        Command::new("rsync")
            .arg("-a")
            .arg("--delete")
            .arg("--compress")
            .arg("--info=progress2")
            .arg(format!("{}:{}/target/", build_server, build_path))
            .arg(format!("{}/target/", project_dir.to_string_lossy()))
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .stdin(Stdio::inherit())
            .output()
            .map_err(Error::TransferFilesError)?;
    }

    Ok(())
}