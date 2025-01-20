use std::path::PathBuf;

use toml::{map::Map, Table, Value};

use crate::{cmake::CMake, manifest::Manifest};

pub struct ManifestMetadata {
    path: PathBuf,
    cmake_hash: Vec<u8>,
    manifest_hash: Vec<u8>,
}

impl ManifestMetadata {
    const FILENAME: &'static str = "TailorMetadata.toml";

    pub fn empty(base_dir: &PathBuf) -> Self {
        Self {
            path: base_dir.join(Self::FILENAME),
            cmake_hash: vec![0u8; 32],
            manifest_hash: vec![0u8; 32],
        }
    }

    pub fn from_folder(base_dir: &PathBuf) -> Result<Self, String> {
        let manifest_metadata_content = std::fs::read_to_string(base_dir.join(Self::FILENAME))
            .map_err(|err| format!("Failed to read {}: {}", Self::FILENAME, err))?;
        let manifest_metadata_toml = manifest_metadata_content
            .parse::<Table>()
            .map_err(|err| format!("Invalid {} format: {}", Self::FILENAME, err))?;
        let path = manifest_metadata_toml
            .get("path")
            .ok_or(format!("Failed to get path from {}", Self::FILENAME))?
            .as_str()
            .ok_or("Path must be a string".to_string())?;

        Ok(Self {
            path: PathBuf::from(path),
            cmake_hash: Self::parse_hash(&manifest_metadata_toml, "cmake_hash")?,
            manifest_hash: Self::parse_hash(&manifest_metadata_toml, "manifest_hash")?,
        })
    }

    fn parse_hash(toml: &Map<String, Value>, attribute: &str) -> Result<Vec<u8>, String> {
        let attr = toml
            .get(attribute)
            .ok_or(format!("Attribute {} not found", attribute))?
            .as_str()
            .ok_or(format!("{} must be a string", attribute))?;

        hex::decode(attr).map_err(|err| format!("Failed to parse hash: {}", err))
    }

    pub fn is_manifest_valid(&self, manifest: &Manifest) -> bool {
        self.manifest_hash == manifest.hash()
    }

    pub fn is_cmake_valid(&self, cmake: &CMake) -> bool {
        self.cmake_hash == cmake.hash()
    }

    pub fn is_path_valid(&self, base_dir: &PathBuf) -> bool {
        self.path == base_dir.join(Self::FILENAME)
    }

    pub fn update_cmake_hash(&mut self, hash: Vec<u8>) {
        self.cmake_hash = hash;
    }

    pub fn update_manifest_hash(&mut self, hash: Vec<u8>) {
        self.manifest_hash = hash;
    }

    pub fn flush(&self) -> Result<(), String> {
        let mut toml = Table::new();
        toml.insert(
            "cmake_hash".to_string(),
            Value::String(hex::encode(&self.cmake_hash)),
        );
        toml.insert(
            "manifest_hash".to_string(),
            Value::String(hex::encode(&self.manifest_hash)),
        );
        toml.insert(
            "path".to_string(),
            Value::String(self.path.to_string_lossy().to_string()),
        );

        std::fs::write(&self.path, toml.to_string())
            .map_err(|err| format!("Failed to write {}: {}", Self::FILENAME, err))
    }
}
