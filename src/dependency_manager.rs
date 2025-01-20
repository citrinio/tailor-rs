use std::path::PathBuf;

use crate::manifest::Dependency;

pub struct DependencyManager;

impl DependencyManager {
    pub fn new() -> Result<Self, String> {
        Ok(Self)
    }

    pub fn fetch(&self, dependencies: &[Dependency]) -> Result<(), String> {
        Ok(())
    }

    pub fn get_sources(&self, dependency_name: &str) -> Vec<PathBuf> {
        vec![]
    }

    pub fn get_includes(&self, dependency_name: &str) -> Vec<PathBuf> {
        vec![]
    }
}
