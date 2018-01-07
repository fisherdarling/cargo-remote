extern crate structopt;
#[macro_use]
extern crate structopt_derive;

extern crate cargo_metadata;

#[macro_use] extern crate log;
extern crate simple_logger;

extern crate toml;

use std::process::{exit, Command, Stdio};
use std::ffi::OsString;
use std::path::Path;
use std::fs::File;
use std::io::{BufReader, Read};

use structopt::StructOpt;

use toml::Value;

#[derive(StructOpt, Debug)]
#[structopt(name = "cargo remote")]
struct Opts {
    // workaround for "remote" argument when calling "cargo remote"
    _unused: String,

    #[structopt(help = "cargo command that will be executed remotely")]
    command: String,

    #[structopt(short = "r", long = "remote", help = "remote ssh build server")]
    remote: Option<String>,

    #[structopt(short = "c", long = "copy-back", help = "transfer the target folder back to the local machine")]
    copy_back: bool
}

fn main() {
    simple_logger::init().unwrap();

    let options = Opts::from_args();
    // TODO: add manifest_path option
    let project_metadata = cargo_metadata::metadata(None).unwrap_or_else(|e| {
        error!("Could not read cargo metadata: {}", e);
        exit(-1);
    });

    // for now, assume that there is only one project and find it's root directory
    let (project_dir, project_name) = project_metadata.packages.first().map_or_else(|| {
        error!("No project found.");
        exit(-2);
    }, |project| {
        (
            Path::new(&project.manifest_path)
                .parent()
                .expect("Cargo.toml seems to have no parent directory?"),
            &project.name
        )
    });

    let build_server = options.remote.unwrap_or_else(|| {
        let config_path = project_dir.join(".cargo-remote.toml");
        File::open(config_path).ok().and_then(|mut file| {
            let mut config_file_string = "".to_owned();
            file.read_to_string(&mut config_file_string);
            config_file_string.parse::<Value>().ok()
        }).and_then(|value| {
            value["remote"].as_str().map(str::to_owned)
        }).unwrap_or_else(|| {
            error!("No remote build server was defined (use config file or --remote flag)");
            exit(-3);
        })
    });

    info!("Transferring sources to build server.");
    // transfer project to build server
    Command::new("rsync")
        .arg("-a")
        .arg("--delete")
        .arg("--info=progress2")
        .arg("--exclude")
        .arg("target")
        .arg(format!("{}/", project_dir.to_string_lossy()))
        .arg(format!("{}:~/remote-builds/{}/", build_server, project_name))
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .stdin(Stdio::inherit())
        .output()
        .expect("failed to transfer project to build server");

    let build_command = format!("cd ~/remote-builds/{}/; $HOME/.cargo/bin/cargo {}", project_name, options.command);

    info!("Starting build process.");
    Command::new("ssh")
        .arg(&build_server)
        .arg(build_command)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .stdin(Stdio::inherit())
        .output()
        .expect("failed to build project");

    if options.copy_back {
        info!("Transferring artifacts back to client.");
        Command::new("rsync")
            .arg("-a")
            .arg("--delete")
            .arg("--info=progress2")
            .arg(format!("{}:~/remote-builds/{}/target/", build_server, project_name))
            .arg(format!("{}/target/", project_dir.to_string_lossy()))
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .stdin(Stdio::inherit())
            .output()
            .expect("failed to transfer built project to client");
    }
}