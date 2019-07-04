use std::path::PathBuf;
use structopt::StructOpt;

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
