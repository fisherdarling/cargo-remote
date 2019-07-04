use structopt::StructOpt;
use cargo_remote::{Config, run_config};
/// Tries to parse the file [`config_path`]. Logs warnings and returns [`None`] if errors occur
/// during reading or parsing, [`Some(Value)`] otherwise.


fn main() {
    let config = Config::from_args();

    std::process::exit(match run_config(config) {
       Ok(_) => 0,
       Err(err) => {
           eprintln!("Encountered an error: {}", err);
           1
       }
    });
}
