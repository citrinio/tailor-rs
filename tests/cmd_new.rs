use assert_cmd::prelude::*;
use chrono::{Datelike, Local};
use predicates::prelude::*;
use rand::Rng;
use std::{error::Error, path::PathBuf, process::Command};
use test_context::{test_context, TestContext};

struct Fixture {
    base_dir: PathBuf,
}

impl TestContext for Fixture {
    fn setup() -> Self {
        let mut rng = rand::thread_rng();
        let rand = rng.gen_range(0..=u32::MAX);

        Fixture {
            base_dir: PathBuf::from(format!("tmp{}", rand)),
        }
    }

    fn teardown(self) {
        let _ = std::fs::remove_dir_all(self.base_dir);
    }
}

mod binary {
    pub const MAIN: &'static str = "#include <stdio.h>

int main(int argc, char *argv[]) {
    printf(\"Hello, World\\n\");

    return 0;
}
";
    pub const MANIFEST: &'static str = "[package]
name = \"hello\"
version = \"0.1.0\"
edition = \"2025.1\"
";
}

pub mod library {
    pub const LIB_SOURCE: &'static str =
        "/** ----------------------------------------------------------------------------
 *  @file hello.c
 *  @brief
 *
 *  @author    John Doe <john.doe@example.com>
 *  @version   v1.0
 *  @date      $date
 *  @copyright Copyright (c)
 *  ----------------------------------------------------------------------------*/
#include \"hello/hello.h\"

int hello_sum(int a, int b) {
    return a + b;
}
";

    pub const LIB_HEADER: &'static str =
        "/** ----------------------------------------------------------------------------
 *  @file hello.h
 *  @brief
 *
 *  @author    John Doe <john.doe@example.com>
 *  @version   v1.0
 *  @date      $date
 *  @copyright Copyright (c)
 *  ----------------------------------------------------------------------------*/
#ifndef HELLO_H
#define HELLO_H

int hello_sum(int a, int b);

#endif /* HELLO_H */
";

    pub const MANIFEST: &'static str = "[package]
name = \"hello\"
version = \"0.1.0\"
edition = \"2025.1\"
type = \"lib\"
";
}

#[test_context(Fixture)]
#[test]
fn test_create_package_with_invalid_package_name(
    fixture: &mut Fixture,
) -> Result<(), Box<dyn Error>> {
    let mut cmd = Command::cargo_bin("tailor")?;

    let package_name = "1hello";
    let package_base_dir = fixture.base_dir.join(package_name);

    cmd.arg("new").arg(&package_base_dir);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains(format!(
            "Invalid package name {}",
            package_name
        )));

    Ok(())
}

#[test_context(Fixture)]
#[test]
fn test_create_package_with_all_flags(fixture: &mut Fixture) -> Result<(), Box<dyn Error>> {
    let mut cmd = Command::cargo_bin("tailor")?;

    let package_name = "hello";
    let package_base_dir = fixture.base_dir.join(package_name);

    cmd.arg("new")
        .arg("--bin")
        .arg("--lib")
        .arg(&package_base_dir);
    cmd.assert().failure().stderr(predicate::str::contains(
        "You need to choose only one package template kind: bin or lib",
    ));

    Ok(())
}

#[test_context(Fixture)]
#[test]
fn test_create_package_with_bin_and_lib_flags(fixture: &mut Fixture) -> Result<(), Box<dyn Error>> {
    let mut cmd = Command::cargo_bin("tailor")?;

    let package_name = "hello";
    let package_base_dir = fixture.base_dir.join(package_name);

    cmd.arg("new")
        .arg("--bin")
        .arg("--lib")
        .arg(&package_base_dir);
    cmd.assert().failure().stderr(predicate::str::contains(
        "You need to choose only one package template kind: bin or lib",
    ));

    Ok(())
}

#[test_context(Fixture)]
#[test]
fn test_create_bin_package_without_flag(fixture: &mut Fixture) -> Result<(), Box<dyn Error>> {
    let mut cmd = Command::cargo_bin("tailor")?;

    let package_name = "hello";
    let package_base_dir = fixture.base_dir.join(package_name);
    let package_manifest_path = package_base_dir.join("Tailor.toml");
    let package_src_path = package_base_dir.join("src");
    let package_main_path = package_base_dir.join("src/main.c");

    cmd.arg("new").arg(&package_base_dir);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(format!(
            "    \x1b[1;32mCreating\x1b[0m binary (application) `{}` package",
            package_name
        )));

    assert!(std::fs::exists(package_base_dir).unwrap());
    assert!(std::fs::exists(&package_manifest_path).unwrap());
    assert!(std::fs::exists(package_src_path).unwrap());
    assert!(std::fs::exists(&package_main_path).unwrap());

    let main_contents = std::fs::read_to_string(package_main_path).unwrap();
    assert_eq!(&main_contents, binary::MAIN);

    let manifest_contents = std::fs::read_to_string(package_manifest_path).unwrap();
    assert_eq!(&manifest_contents, binary::MANIFEST);

    Ok(())
}

#[test_context(Fixture)]
#[test]
fn test_create_bin_package_with_flag(fixture: &mut Fixture) -> Result<(), Box<dyn Error>> {
    let mut cmd = Command::cargo_bin("tailor")?;

    let package_name = "hello";
    let package_base_dir = fixture.base_dir.join(package_name);
    let package_manifest_path = package_base_dir.join("Tailor.toml");
    let package_src_path = package_base_dir.join("src");
    let package_main_path = package_base_dir.join("src/main.c");

    cmd.arg("new").arg("--bin").arg(&package_base_dir);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(format!(
            "    \x1b[1;32mCreating\x1b[0m binary (application) `{}` package",
            package_name
        )));

    assert!(std::fs::exists(package_base_dir).unwrap());
    assert!(std::fs::exists(&package_manifest_path).unwrap());
    assert!(std::fs::exists(package_src_path).unwrap());
    assert!(std::fs::exists(&package_main_path).unwrap());

    let main_contents = std::fs::read_to_string(package_main_path).unwrap();
    assert_eq!(&main_contents, binary::MAIN);

    let manifest_contents = std::fs::read_to_string(package_manifest_path).unwrap();
    assert_eq!(&manifest_contents, binary::MANIFEST);

    Ok(())
}

#[test_context(Fixture)]
#[test]
fn test_create_lib_package(fixture: &mut Fixture) -> Result<(), Box<dyn Error>> {
    let date = Local::now();
    let date_str = format!("{:02}/{:02}/{:04}", date.month(), date.day(), date.year());
    let mut cmd = Command::cargo_bin("tailor")?;

    let package_name = "hello";
    let package_base_dir = fixture.base_dir.join(package_name);
    let package_manifest_path = package_base_dir.join("Tailor.toml");
    let package_src_path = package_base_dir.join("src");
    let package_source_path = package_base_dir.join("src/hello.c");
    let package_header_path = package_base_dir.join("src/hello/hello.h");

    cmd.arg("new").arg("--lib").arg(&package_base_dir);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(format!(
            "    \x1b[1;32mCreating\x1b[0m library `{}` package",
            package_name
        )));

    assert!(std::fs::exists(package_base_dir).unwrap());
    assert!(std::fs::exists(&package_manifest_path).unwrap());
    assert!(std::fs::exists(package_src_path).unwrap());
    assert!(std::fs::exists(&package_source_path).unwrap());
    assert!(std::fs::exists(&package_header_path).unwrap());

    let source_contents = std::fs::read_to_string(package_source_path).unwrap();
    assert_eq!(
        source_contents,
        library::LIB_SOURCE.replace("$date", &date_str)
    );

    let header_contents = std::fs::read_to_string(package_header_path).unwrap();
    assert_eq!(
        header_contents,
        library::LIB_HEADER.replace("$date", &date_str)
    );

    let manifest_contents = std::fs::read_to_string(package_manifest_path).unwrap();
    assert_eq!(&manifest_contents, library::MANIFEST);

    Ok(())
}
