use std::path::PathBuf;

use clap::Parser;
use clap::ValueEnum;

use iceoryx2_cli::help_template;
use iceoryx2_cli::HelpOptions;

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Lang {
    Rust,
    Cpp,
    Python,
}

#[derive(Parser)]
#[command(
    name = "iox2 payload-gen",
    bin_name = "iox2 payload-gen",
    about = "Generate iceoryx2-compatible payload types from ROS interface files",
    long_about = None,
    version = env!("CARGO_PKG_VERSION"),
    disable_help_subcommand = true,
    arg_required_else_help = true,
    help_template = help_template(HelpOptions::DontPrintCommandSection),
)]
pub struct Cli {
    /// Target language: rust, cpp, python
    #[arg(short, long)]
    pub lang: Lang,

    /// Output path:
    /// - file input: file path or directory path
    /// - directory input: directory path only
    #[arg(short, long, default_value = ".")]
    pub output: PathBuf,

    /// Optional prefix prepended to every generated service name, e.g. "ros2/"
    #[arg(long, default_value = "")]
    pub service_prefix: String,

    /// Input interface file (.msg/.srv) or directory containing them
    #[arg(required = true)]
    pub input: PathBuf,
}
