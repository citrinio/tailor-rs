use std::path::PathBuf;

use chrono::Datelike;
use clap::{Parser, Subcommand};
use regex::Regex;
use tailor::templates;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    New {
        path: PathBuf,
        #[clap(long, action, help = "Use a binary (application) template [default]")]
        bin: bool,
        #[clap(long, action, help = "Use a library template")]
        lib: bool,
    },
}

fn main() -> Result<(), String> {
    let cli = Cli::parse();

    match cli.command {
        Command::New { path, bin, lib } => {
            let flags = [bin, lib].into_iter().filter(|&v| v).collect::<Vec<_>>();

            if flags.len() > 1 {
                return Err(
                    "You need to choose only one package template kind: bin or lib".to_string(),
                );
            }

            let path2 = path.clone();
            let mut path_components = path.components();
            let package_name = path_components
                .next_back()
                .unwrap()
                .as_os_str()
                .to_str()
                .unwrap();

            if !is_package_name_valid(package_name) {
                return Err(format!("Invalid package name {}", package_name));
            }

            return if lib {
                create_lib_package(package_name, path2)
            } else {
                create_bin_package(package_name, path2)
            };
        }
    }
}

fn is_package_name_valid(package_name: &str) -> bool {
    let re = Regex::new("^[_a-zA-Z][_a-zA-Z0-9]*").unwrap();
    re.is_match(package_name)
}

fn create_bin_package(package_name: &str, path: PathBuf) -> Result<(), String> {
    let manifest = templates::binary::MANIFEST.replace("$name", package_name);

    std::fs::create_dir_all(path.join("src"))
        .map_err(|err| format!("Failed to create directories: {}", err))?;
    std::fs::write(path.join("Tailor.toml"), manifest)
        .map_err(|err| format!("Failed to write manifest file: {}", err))?;
    std::fs::write(path.join("src/main.c"), templates::binary::MAIN)
        .map_err(|err| format!("Failed to write main.c file: {}", err))?;

    println!(
        "    \x1b[1;32mCreating\x1b[0m binary (application) `{}` package",
        package_name
    );

    Ok(())
}

fn create_lib_package(package_name: &str, path: PathBuf) -> Result<(), String> {
    let datetime = chrono::Local::now();
    let date_str = format!(
        "{:02}/{:02}/{:04}",
        datetime.month(),
        datetime.day(),
        datetime.year()
    );
    let manifest = templates::library::MANIFEST.replace("$name", package_name);
    let lib_source = templates::library::LIB_SOURCE
        .replace("$name", package_name)
        .replace("$date", &date_str);
    let lib_header = templates::library::LIB_HEADER
        .replace("$nameup", &package_name.to_uppercase())
        .replace("$name", package_name)
        .replace("$date", &date_str);

    std::fs::create_dir_all(path.join(format!("src/{}", package_name)))
        .map_err(|err| format!("Failed to create directories: {}", err))?;
    std::fs::write(path.join("Tailor.toml"), manifest)
        .map_err(|err| format!("Failed to write manifest file: {}", err))?;
    std::fs::write(path.join(format!("src/{}.c", package_name)), lib_source)
        .map_err(|err| format!("Failed to write {}.c file: {}", package_name, err))?;
    std::fs::write(
        path.join(format!("src/{}/{}.h", package_name, package_name)),
        lib_header,
    )
    .map_err(|err| format!("Failed to write {}.h file: {}", package_name, err))?;

    println!(
        "    \x1b[1;32mCreating\x1b[0m library `{}` package",
        package_name
    );

    Ok(())
}
