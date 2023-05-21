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
    pub kvm: bool,
    pub uefi: bool,
}

impl Default for Machine {
    fn default() -> Self {
        Machine {
            disk: "./disk.qcow2".into(),
            cpus: 2,
            memory: 2048,
            kvm: true,
            uefi: false,
        }
    }
}

impl Config {
    pub fn construct_launch_command<P: AsRef<Path>>(
        &self,
        containing_dir_path: P,
        cd_rom: Option<&Path>,
    ) -> Command {
        let Config {
            machine:
                Machine {
                    disk,
                    cpus,
                    memory,
                    kvm,
                    uefi,
                },
        } = self;

        let mut command = Command::new("qemu-system-x86_64");

        let disk_path = if disk.is_absolute() {
            disk.clone()
        } else {
            containing_dir_path.as_ref().join(disk)
        };

        let mut args = Vec::new();

        let binding = cpus.to_string();
        args.extend(["-smp", binding.as_str()]);
        let binding = format!("{memory}M");
        args.extend(["-m", binding.as_str()]);

        if *kvm {
            args.push("-enable-kvm");
        }

        if *uefi {
            args.extend([
                "-drive",
                "if=pflash,format=raw,readonly=on,file=/usr/share/edk2-ovmf/x64/OVMF_CODE.fd",
            ]);
        }

        let cd_rom = cd_rom.map(|p| p.to_string_lossy().to_string());
        if let Some(cd_rom) = &cd_rom {
            args.extend(["-cdrom", cd_rom])
        }

        let binding = disk_path.to_string_lossy();
        args.push(binding.as_ref());

        command.args(args);

        command
    }
}
