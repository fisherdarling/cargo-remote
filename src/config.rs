use std::path::{PathBuf, Path};
use std::process::Stdio;
use structopt::StructOpt;

use std::process::Command;

#[derive(StructOpt, Debug)]
#[structopt(name = "cargo-remote", bin_name = "cargo")]
pub enum Config {
    #[structopt(name = "remote")]
    Remote {
        /// address of the remote ssh build server
        #[structopt(short = "r", long = "remote")]
        remote: Option<String>,

        /// transfer the target folder back to the local machine
        #[structopt(short = "c", long = "copy-back")]
        copy_back: bool,

        /// Path to the manifest to execute
        #[structopt(
            long = "manifest-path",
            default_value = "Cargo.toml",
            parse(from_os_str)
        )]
        manifest_path: PathBuf,

        /// transfer hidden files and directories to the build server
        #[structopt(short = "h", long = "transfer-hidden")]
        hidden: bool,

        /// cargo command that will be executed remotely
        #[structopt(name = "cargo command")]
        command: String,

        /// cargo options and flags that will be applied remotely
        #[structopt(name = "remote options")]
        options: Vec<String>,
    },
}


impl Config {
    pub fn rsync_to(&self, project_dir: &Path, build_server: &str, build_path: &str) -> Command {
        let Config::Remote { hidden, .. } = self;
        
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

        rsync_to
    }
}