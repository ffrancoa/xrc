use clap::builder::styling::{AnsiColor, Effects, Styles};
use clap::{Parser, Subcommand};

fn styles() -> Styles {
    Styles::styled()
        .header(AnsiColor::Green.on_default().effects(Effects::BOLD))
        .usage(AnsiColor::Green.on_default().effects(Effects::BOLD))
        .literal(AnsiColor::Cyan.on_default())
        .placeholder(AnsiColor::Cyan.on_default())
        .valid(AnsiColor::Green.on_default().effects(Effects::BOLD))
        .invalid(AnsiColor::Yellow.on_default().effects(Effects::BOLD))
        .error(AnsiColor::Red.on_default().effects(Effects::BOLD))
}

/// Give your Rust some exercise.
#[derive(Parser)]
#[command(name = "xrc", version, styles = styles(), arg_required_else_help = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Create a Rust project from problem URL.
    Pull {
        /// Problem page URL (dmoj.ca or leetcode.com)
        url: String,
    },
    /// Check the status of all exercises in the exercises directory.
    Check {
        /// Re-run all exercises regardless of saved state
        #[arg(long)]
        recheck: bool,
        /// Show detailed build/test output
        #[arg(long)]
        verbose: bool,
    },
}
