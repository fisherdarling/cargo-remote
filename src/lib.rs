// use std::fs::PathBuf;
use std::path::{Path, PathBuf};
use std::process::{exit, Command, Stdio};

use toml::Value;

use log::{error, info, warn};

use structopt::StructOpt;

pub mod utils;
pub mod config;
pub mod error;


pub use config::Config;

use error::Error;

pub fn run_config(config: &Config) -> Result<(), Error> {
    let Config::Remote {
        remote,
        copy_back,
        manifest_path,
        hidden,
        command,
        options,
    } = Config::from_args();

    let project_metadata = utils::get_project_metadata(manifest_path)?;

    // for now, assume that there is only one project and find it's root directory
    let (project_dir, project_name) = project_metadata.packages.first().map_or_else(
        || {
            error!("No project found.");
            exit(-2);
        },
        |project| {
            (
                &project
                    .manifest_path
                    .parent()
                    .ok_or(Error::CargoTomlNoParentError)?,
                &project.name,
            )
        },
    );

    let configs = vec![
        config_from_file(&project_dir.join(".cargo-remote.toml")),
        xdg::BaseDirectories::with_prefix("cargo-remote")
            .ok()
            .and_then(|base| base.find_config_file("cargo-remote.toml"))
            .and_then(|p: PathBuf| config_from_file(&p)),
    ];

    // TODO: move Opts::Remote fields into own type and implement complete_from_config(&mut self, config: &Value)
    let build_server = remote
        .or_else(|| {
            configs
                .into_iter()
                .flat_map(|config| config.and_then(|c| c["remote"].as_str().map(String::from)))
                .next()
        })
        .unwrap_or_else(|| {
            error!("No remote build server was defined (use config file or --remote flag)");
            exit(-3);
        });

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
        .stdin(Stdio::inherit())
        .output()
        .unwrap_or_else(|e| {
            error!("Failed to transfer project to build server (error: {})", e);
            exit(-4);
        });

    let build_command = format!(
        "cd {}; $HOME/.cargo/bin/cargo {} {}",
        build_path
        project_name,
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
        .unwrap_or_else(|e| {
            error!("Failed to run cargo command remotely (error: {})", e);
            exit(-5);
        });

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
            .unwrap_or_else(|e| {
                error!(
                    "Failed to transfer target back to local machine (error: {})",
                    e
                );
                exit(-6);
            });
    }
}