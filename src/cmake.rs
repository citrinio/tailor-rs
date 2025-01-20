use crate::{dependency_manager::DependencyManager, manifest::Manifest, templates};
use sha2::{Digest, Sha256};
use std::{path::PathBuf, process::Stdio};

pub struct CMake {
    build_path: PathBuf,
    path: PathBuf,
    cmake_contents: String,
}

impl CMake {
    pub fn new_binary(
        base_dir: &PathBuf,
        manifest: &Manifest,
        manifest_dir: &PathBuf,
        dependency_manager: &DependencyManager,
    ) -> Result<Self, String> {
        let path = base_dir.join("CMakeLists.txt");
        let mut sources = manifest
            .src()
            .iter()
            .map(|src| manifest_dir.join(src))
            .collect::<Vec<_>>();
        let mut includes = manifest
            .include()
            .iter()
            .map(|inc| manifest_dir.join(inc))
            .collect::<Vec<_>>();

        for dependency in manifest.dependencies() {
            let dependency_name = dependency.name();

            sources.extend(dependency_manager.get_sources(&dependency_name));
            includes.extend(dependency_manager.get_includes(&dependency_name));
        }

        let sources_str = sources
            .into_iter()
            .map(|src| format!("\t{:?}", src))
            .collect::<Vec<_>>()
            .join("\n");
        let includes_str = includes
            .into_iter()
            .map(|inc| format!("\t{:?}", inc))
            .collect::<Vec<_>>()
            .join("\n");

        let cmake_contents = templates::binary::CMAKE
            .replace("$name", manifest.name())
            .replace("$src", &sources_str)
            .replace("$include", &includes_str);

        Ok(Self {
            build_path: base_dir.clone(),
            path,
            cmake_contents,
        })
    }

    pub fn flush(&self) -> Result<(), String> {
        std::fs::write(&self.path, &self.cmake_contents)
            .map_err(|err| format!("Failed to create cmake for binary package: {}", err))
    }

    pub fn hash(&self) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(&self.cmake_contents);
        hasher.finalize().to_vec()
    }

    pub fn build(&self, jobs: u8, is_release: bool) -> Result<(), String> {
        let output = std::process::Command::new("cmake")
            .arg("-GNinja")
            .arg(&format!("-S{}", self.build_path.to_string_lossy()))
            .arg(&format!("-B{}", self.build_path.to_string_lossy()))
            .arg(if is_release {
                "-DCMAKE_BUILD_TYPE=Release"
            } else {
                "-DCMAKE_BUILD_TYPE=Debug"
            })
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|err| format!("Failed to run cmake command: {}", err))?;

        if !output.status.success() {
            return Err(std::str::from_utf8(output.stderr.as_slice())
                .map_err(|err| format!("Failed to get stderr: {}", err))?
                .to_string());
        }

        let output = std::process::Command::new("ninja")
            .arg("-C")
            .arg(&self.build_path)
            .arg("-j")
            .arg(&format!("{}", jobs))
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|err| format!("Failed to run ninja command: {}", err))?;

        if !output.status.success() {
            return Err(std::str::from_utf8(output.stderr.as_slice())
                .map_err(|err| format!("Failed to get stderr: {}", err))?
                .to_string());
        }

        Ok(())
    }
}
