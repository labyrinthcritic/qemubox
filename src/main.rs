mod cli;
mod config;

use std::{
    fs::ReadDir,
    path::PathBuf,
    process::{Command, Stdio},
};

use colored::Colorize;
use config::Config;

use cli::{CliCommand, MachineCommand};

fn main() {
    let cli = <cli::Cli as clap::Parser>::parse();

    let result = match cli.command {
        CliCommand::Ls => ls(),
        CliCommand::New { name, disk_size } => new_machine(name, disk_size),
        CliCommand::Machine { name, cmd } => machine(name, cmd),
    };

    if let Err(e) = result {
        report_error(e);
    }
}

// cli command implementations

/// Handling of the `Cli::Ls` subcommand.
fn ls() -> Result<(), Error> {
    let (_, machines) = get_machines()?;

    if machines.is_empty() {
        println!("{}", "No machines found.".bold());
    } else {
        println!("    {}", "Name".bold());
        for machine in machines {
            println!("    {}", machine.name);
        }
    }

    Ok(())
}

/// Handling of the `Cli::Machine` subcommand.
fn machine(name: String, cmd: MachineCommand) -> Result<(), Error> {
    match get_machines() {
        Ok((machines_dir, machines)) => {
            if let Some(machine) = machines.iter().find(|m| m.name == name) {
                match cmd {
                    cli::MachineCommand::Run => {
                        println!("Launching {}...", machine.name.bright_green().bold());
                        machine
                            .config
                            .construct_launch_command(machines_dir.join(&machine.name))
                            .stdout(Stdio::piped())
                            .spawn()
                            .unwrap()
                            .wait()
                            .unwrap();
                    }
                    cli::MachineCommand::Remove { yes } => {
                        let dir_to_remove = machines_dir.join(&machine.name);
                        if yes {
                            std::fs::remove_dir_all(&dir_to_remove).unwrap();
                        } else {
                            display_warning(format!("are you sure? This will remove {dir_to_remove:?} and all of its contents.\nRun with --yes to confirm.").as_str());
                        }
                    }
                }
            } else {
                return Err(Error::NoMachineByName);
            }
        }
        Err(_) => todo!(),
    }

    Ok(())
}

/// This structure represents an instance of a machine found in the machines directory.
/// These are constructed by [`get_machines`].
struct Machine {
    name: String,
    config: Config,
}

fn get_machines_dir() -> Result<(PathBuf, ReadDir), Error> {
    let Some(home) = dirs::home_dir() else { return Err(Error::NoHomeDirectory); };
    let machines_dir = home.join(".local/share/qemubox/machines");
    let Ok(read_machines_dir) = std::fs::read_dir(home.join(".local/share/qemubox/machines"))
        else { return Err(Error::ReadMachinesDirectoryFail); };

    Ok((machines_dir, read_machines_dir))
}

/// Search the machines directory and get its path, alongside [`Machine`]s.
fn get_machines() -> Result<(PathBuf, Vec<Machine>), Error> {
    let (machines_dir, read_machines_dir) = get_machines_dir()?;
    let mut machines = Vec::new();

    for item in read_machines_dir {
        if let Ok(item) = item {
            if let Ok(metadata) = item.metadata() {
                if metadata.is_dir() {
                    let config_path = item.path().join("machine.toml");
                    if let Ok(config_text) = std::fs::read_to_string(&config_path) {
                        match toml::from_str::<Config>(&config_text) {
                            Ok(config) => machines.push(Machine {
                                name: item.path().file_name().unwrap().to_string_lossy().into(),
                                config,
                            }),
                            Err(e) => return Err(Error::Deserialize(config_path, e)),
                        }
                    }
                }
            }
        }
    }

    Ok((machines_dir, machines))
}

/// Create a new machine in the machines directory with a name and disk size.
fn new_machine(name: String, disk_size: u32) -> Result<(), Error> {
    let (machines_dir, read_machines_dir) = get_machines_dir()?;
    for machine in read_machines_dir {
        if let Ok(machine) = machine {
            if machine.file_name().to_string_lossy() == name {
                return Err(Error::NewMachineDirectoryExists);
            }
        }
    }

    let machine_dir = machines_dir.join(name);
    if std::fs::create_dir(&machine_dir).is_err() {
        return Err(Error::CreateMachineDirectoryFail);
    }

    let config = toml::to_string_pretty(&Config::default()).unwrap();

    if std::fs::write(machine_dir.join("machine.toml"), config).is_err() {
        return Err(Error::WriteMachineTomlFail);
    }

    // create the disk

    Command::new("qemu-img")
        .args([
            "create",
            "-f",
            "qcow2",
            machine_dir.join("disk.qcow2").to_string_lossy().as_ref(),
            format!("{disk_size}M").as_str(),
        ])
        .stdout(Stdio::null())
        .spawn()
        .unwrap();

    Ok(())
}

enum Error {
    NoHomeDirectory,
    ReadMachinesDirectoryFail,
    Deserialize(PathBuf, toml::de::Error),
    NewMachineDirectoryExists,
    CreateMachineDirectoryFail,
    WriteMachineTomlFail,
    NoMachineByName,
}

fn report_error(error: Error) {
    match error {
        Error::NoHomeDirectory => display_error("you have no home directory"),
        Error::ReadMachinesDirectoryFail => {
            display_error("could not read ~/.local/share/qemubox/machines")
        }
        Error::Deserialize(config_path, e) => {
            display_error(format!("in reading {config_path:?},\n    {:?}", e.message()).as_str())
        }
        Error::NewMachineDirectoryExists => {
            display_error("a machine directory with that name already exists")
        }
        Error::CreateMachineDirectoryFail => {
            display_error("could not create the directory for the machine")
        }
        Error::WriteMachineTomlFail => display_error("could not write machines.toml"),
        Error::NoMachineByName => display_error("no machine found by that name"),
    }
}

fn display_error(error: &str) {
    eprintln!("{}{} {error}", "error".bright_red().bold(), ":".bold());
}

fn display_warning(warning: &str) {
    eprintln!(
        "{}{} {warning}",
        "warning".bright_yellow().bold(),
        ":".bold()
    );
}
