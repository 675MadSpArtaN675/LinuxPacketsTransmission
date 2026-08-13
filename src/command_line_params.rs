pub use clap::Parser;
use clap::{Args, Subcommand};

use std::fmt::Debug;

#[derive(Parser, Debug)]
#[command(name = "pac_transmission", version = "0.1", about, long_about = None)]
pub struct AutomatonArgs {
    #[command(subcommand)]
    pub subcommands: RecieveCommand,
}

#[derive(Subcommand, Debug)]
pub enum RecieveCommand {
    Send(SendArgs),
    Recieve(RecieveArgs)
}


#[derive(Args, Debug)]
pub struct SendArgs {
    #[arg(short, long, default_value_t = "zypper".to_string())]
    pub name: String,

    pub ip_to_send: Vec<String>,

    #[arg(short = 's', long)]
    pub path_to_save: Option<String>,

    #[arg(long, default_value_t = 2020)]
    pub port: u32,
}

#[derive(Args, Debug)]
pub struct RecieveArgs {
    #[arg(short, long, default_value_t = "zypper".to_string())]
    pub name: String,

    #[arg(short = 'r', long, default_value_t = true)]
    pub is_install_repositories: bool,

    #[arg(short = 'p', long, default_value_t = true)]
    pub is_install_packages: bool,

    #[arg(short = 's', long)]
    pub path_to_save: Option<String>,

    #[arg(long, default_value_t = 2020)]
    pub port: u32
}