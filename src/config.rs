use std::{
    path::{Path, PathBuf},
    process::Command,
};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub machine: Machine,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Machine {
    pub disk: PathBuf,
    pub cpus: u32,
    pub memory: u32,
}

impl Default for Machine {
    fn default() -> Self {
        Machine {
            disk: "./disk.qcow2".into(),
            cpus: 2,
            memory: 2048,
        }
    }
}

impl Config {
    pub fn construct_launch_command<P: AsRef<Path>>(&self, containing_dir_path: P) -> Command {
        let Config {
            machine: Machine { disk, cpus, memory },
        } = self;

        let mut command = Command::new("qemu-system-x86_64");

        let disk_path = if disk.is_absolute() {
            disk.clone()
        } else {
            containing_dir_path.as_ref().join(disk)
        };

        command.args([
            "-smp",
            cpus.to_string().as_str(),
            "-m",
            format!("{memory}M").as_str(),
            disk_path.as_os_str().to_string_lossy().as_ref(),
        ]);

        command
    }
}
