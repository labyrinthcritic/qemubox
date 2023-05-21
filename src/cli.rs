use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: CliCommand,
}

#[derive(Subcommand)]
pub enum CliCommand {
    /// List machines.
    Ls,
    /// Create a new machine with a name and disk size.
    New {
        name: String,
        #[arg(long)]
        disk_size: u32,
    },
    /// Run or edit a machine.
    Machine {
        name: String,
        #[command(subcommand)]
        cmd: MachineCommand,
    },
}

#[derive(Subcommand)]
pub enum MachineCommand {
    /// Launch a qemu-system-x86_64 instance for this machine.
    Run {
        #[arg(long)]
        cd_rom: Option<PathBuf>,
        #[arg(short, long)]
        verbose: bool,
    },
    /// Remove the machine's directory.
    Remove {
        #[arg(long)]
        yes: bool,
    },
    /// Open machine.toml in $EDITOR
    Edit,
}
