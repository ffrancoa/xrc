use clap::Parser;

use xrc::cli::{Cli, Command};
use xrc::commands::{cmd_check, cmd_pull};

fn main() {
    let cli = Cli::parse();
    let code = match cli.command {
        Command::Pull { url } => cmd_pull(&url),
        Command::Check { recheck, verbose } => cmd_check(recheck, verbose),
    };
    std::process::exit(code);
}
