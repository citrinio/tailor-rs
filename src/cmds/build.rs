use std::path::PathBuf;

use crate::{
    cmake::CMake, dependency_manager::DependencyManager, manifest::Manifest,
    manifest_metadata::ManifestMetadata,
};

fn make_build_path(pwd: &PathBuf, is_release: bool) -> Result<PathBuf, String> {
    let build_dir_path: PathBuf = pwd.join("build");

    let build_dir_path = if !is_release {
        build_dir_path.join("debug")
    } else {
        build_dir_path.join("release")
    };

    if !std::fs::exists(&build_dir_path)
        .map_err(|err| format!("Failed to check build existence: {}", err))?
    {
        std::fs::create_dir_all(&build_dir_path)
            .map_err(|err| format!("Failed to create build dir: {}", err))?;
    }

    std::fs::canonicalize(build_dir_path)
        .map_err(|err| format!("Failed to get full path for build directory: {}", err))
}

pub fn cmd_build(release: bool, jobs: u8) -> Result<(), String> {
    let pwd =
        std::env::current_dir().map_err(|err| format!("Failed to get current dir: {}", err))?;
    let build_dir_path = make_build_path(&pwd, release)?;

    let manifest = Manifest::from_folder(&pwd)?;
    let dependency_manager = DependencyManager::new()?;
    let mut metadata = match ManifestMetadata::from_folder(&build_dir_path) {
        Ok(metadata) => metadata,
        Err(_err) => ManifestMetadata::empty(&build_dir_path),
    };
    let cmake = match manifest.package_type() {
        crate::manifest::PackageType::Binary => {
            CMake::new_binary(&build_dir_path, &manifest, &pwd, &dependency_manager)?
        }
        crate::manifest::PackageType::Library => todo!(),
        crate::manifest::PackageType::SDK => todo!(),
    };

    println!("Fetching dependencies...");
    dependency_manager.fetch(manifest.dependencies())?;

    let need_regen_cmake = metadata.is_manifest_valid(&manifest)
        || metadata.is_cmake_valid(&cmake)
        || metadata.is_path_valid(&build_dir_path);

    if need_regen_cmake {
        println!("Writing CMakeLists.txt...");
        cmake.flush()?;
        println!("Writing TailorMetadata.toml...");
        metadata.update_cmake_hash(cmake.hash());
        metadata.update_manifest_hash(manifest.hash());
        metadata.flush()?;
    }

    println!("Building...");
    cmake.build(jobs, release)?;

    println!("Build complete with success!");
    Ok(())
}
