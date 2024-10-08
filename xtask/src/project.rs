use std::{fs, io::Write, path::PathBuf};

use anyhow::{Ok, Result};
use sparreal_build::ProjectConfig;

use crate::compile::Compile;

pub enum Arch {
    Aarch64,
    Riscv64,
    X86_64,
}

impl Default for Arch {
    fn default() -> Self {
        Self::Aarch64
    }
}

impl Arch {
    pub fn qemu_arch(&self) -> String {
        let arch = match self {
            Arch::Aarch64 => "aarch64",
            Arch::Riscv64 => "riscv64",
            Arch::X86_64 => "x86_64",
        };

        format!("qemu-system-{}", arch)
    }

    fn from_target(target: &str) -> Result<Arch> {
        if target.contains("aarch64") {
            return Ok(Arch::Aarch64);
        }

        if target.contains("riscv64") {
            return Ok(Arch::Riscv64);
        }

        if target.contains("x86_64") {
            return Ok(Arch::X86_64);
        }

        Err(anyhow::anyhow!("Unsupportedtarget: {}", target))
    }
}

#[derive(Default)]
pub struct Project {
    pub config: ProjectConfig,
    pub config_path: PathBuf,
    pub arch: Arch,
    pub compile: Option<Compile>,
}
impl Project {
    pub fn new(config: Option<String>) -> Result<Self> {
        let config = config.unwrap_or(".project.toml".to_string());
        if !fs::exists(&config)? {
            let mut file = fs::File::create(&config)?;
            let config_str = ProjectConfig::default().to_string();
            file.write_all(config_str.as_bytes())?;
        }
        let config_path = std::fs::canonicalize(&config)?;

        let content = std::fs::read_to_string(&config_path)?;

        let config: ProjectConfig = toml::from_str(&content)?;

        let target = &config.build.target;

        let arch = Arch::from_target(&target)?;

        Ok(Project {
            config,
            config_path,
            arch,
            ..Default::default()
        })
    }

    pub fn build(&mut self, debug: bool) -> Result<()> {
        let compile = Compile::run(self, debug)?;
        self.compile = Some(compile);
        Ok(())
    }

    pub fn output_dir(&self, debug: bool) -> PathBuf {
        let pwd = std::fs::canonicalize(".").unwrap();

        let target = &self.config.build.target;

        pwd.join("target")
            .join(target)
            .join(if debug { "debug" } else { "release" })
    }
}
