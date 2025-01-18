use std::path::{Path, PathBuf};

use toml::{map::Map, Table, Value};

#[derive(Debug, Default, PartialEq)]
pub struct Manifest {
    name: String,
    version: String,
    edition: Edition,
    package_type: PackageType,
    dependencies: Vec<Dependency>,
    src: Vec<String>,
    include: Vec<String>,
}

#[derive(Debug, Default, PartialEq)]
pub enum Edition {
    #[default]
    Edition2025_1,
}

#[derive(Debug, Default, PartialEq)]
pub enum PackageType {
    #[default]
    Binary,
    Library,
    SDK,
}

#[derive(Debug, PartialEq)]
pub enum Dependency {
    Package {
        name: String,
        version: String,
    },
    Git {
        name: String,
        repo: String,
        version: Option<String>,
        revision: Option<String>,
    },
    Local {
        name: String,
        path: PathBuf,
    },
}

fn ord(i: usize) -> String {
    match i % 10 {
        1 => "st",
        2 => "nd",
        3 => "rd",
        _ => "th",
    }
    .to_string()
}

impl Manifest {
    pub fn from_folder(folder: &Path) -> Result<Self, String> {
        if !folder.is_dir() {
            return Err(format!("Isn't a directory: {:?}", folder));
        }

        for path in std::fs::read_dir(folder).unwrap() {
            let path = path.unwrap().path();

            if let Some(filename) = path.file_name() {
                if filename == "Tailor.toml" {
                    return Self::from_file(&path);
                }
            }
        }

        return Err("No found Tailor.toml in directory".to_string());
    }

    pub fn from_file(manifest_file: &Path) -> Result<Self, String> {
        let manifest_content = std::fs::read_to_string(manifest_file)
            .map_err(|err| format!("Cannot read file {:?}: {}", manifest_file, err))?;
        let manifest_toml = manifest_content
            .parse::<Table>()
            .map_err(|err| format!("Invalid Manifest format: {}", err))?;

        if !manifest_toml.contains_key("package") {
            return Err("package is missing".to_string());
        }

        if manifest_toml["package"].as_table().is_none() {
            return Err("package is not a table".to_string());
        }

        let manifest = Manifest {
            name: Self::get_package_attribute(&manifest_toml, "name")
                .ok_or(format!("Package name not found"))?,
            version: Self::get_package_attribute(&manifest_toml, "version")
                .ok_or(format!("Package version not found"))?,
            edition: Self::get_package_attribute(&manifest_toml, "edition")
                .ok_or(format!("Package edition not found"))?
                .as_str()
                .try_into()?,
            package_type: Self::get_package_attribute(&manifest_toml, "type")
                .unwrap_or("bin".to_string())
                .as_str()
                .try_into()?,
            dependencies: Self::get_dependencies(&manifest_toml)?,
            src: Self::get_package_list_attributes(&manifest_toml, "src")
                .unwrap_or(Ok(vec!["src/*.c".to_string()]))?,
            include: Self::get_package_list_attributes(&manifest_toml, "include")
                .unwrap_or(Ok(vec!["src/".to_string()]))?,
        };

        Ok(manifest)
    }

    fn get_package_attribute(manifest: &Map<String, Value>, attribute: &str) -> Option<String> {
        manifest["package"]
            .get(attribute)
            .map(|attr| attr.as_str().map(|attr| attr.to_string()))
            .flatten()
    }

    fn get_package_list_attributes(
        manifest: &Map<String, Value>,
        attribute: &str,
    ) -> Option<Result<Vec<String>, String>> {
        manifest["package"].get(attribute).map(|attr| {
            let Some(attr_list) = attr.as_array() else {
                return Err(format!("Package \"{}\" is not a list", attribute));
            };

            let mut output = vec![];

            for (i, attr) in attr_list.iter().enumerate() {
                let Some(attr) = attr.as_str() else {
                    return Err(format!(
                        "{}{} package \"{}\" is not a string",
                        i,
                        ord(i),
                        attribute
                    ));
                };

                output.push(attr.to_string());
            }

            Ok(output)
        })
    }

    fn get_dependencies(manifest: &Map<String, Value>) -> Result<Vec<Dependency>, String> {
        if !manifest.contains_key("dependencies") {
            return Ok(vec![]);
        }

        let Some(dependencies) = manifest["dependencies"].as_table() else {
            return Err("dependencies is not a table".to_string());
        };

        let dependencies = dependencies
            .iter()
            .map(|entry| Dependency::try_from(entry))
            .collect::<Vec<_>>();

        let dependencies_errors = dependencies
            .iter()
            .filter_map(|d| if let Err(e) = d { Some(e) } else { None })
            .collect::<Vec<_>>();

        match dependencies_errors.len() {
            0 => {
                let mut dependencies = dependencies
                    .into_iter()
                    .filter_map(|d| d.ok())
                    .collect::<Vec<_>>();
                dependencies.sort_by_key(|dep| dep.name());
                Ok(dependencies)
            }
            1 => Err(dependencies_errors[0].to_owned()),
            _ => {
                let errors = dependencies_errors
                    .into_iter()
                    .map(|err| err.to_owned())
                    .collect::<Vec<_>>()
                    .join("\n");

                Err(format!("Multiple dependency errors:\n{}", errors))
            }
        }
    }
}

impl Dependency {
    fn name(&self) -> String {
        match self {
            Dependency::Package { name, .. } => name.to_owned(),
            Dependency::Git { name, .. } => name.to_owned(),
            Dependency::Local { name, .. } => name.to_owned(),
        }
    }
}

impl TryFrom<&str> for Edition {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "2025.1" => Ok(Edition::Edition2025_1),
            _ => Err(format!("Invalid edition {}", value)),
        }
    }
}

impl TryFrom<&str> for PackageType {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "bin" => Ok(PackageType::Binary),
            "lib" => Ok(PackageType::Library),
            "sdk" => Ok(PackageType::SDK),
            _ => Err(format!("Invalid package type {}", value)),
        }
    }
}

impl TryFrom<(&String, &Value)> for Dependency {
    type Error = String;

    fn try_from((name, attributes): (&String, &Value)) -> Result<Self, Self::Error> {
        let mut workdir = std::env::current_dir()
            .map_err(|err| format!("Failed to get current directory: {}", err))?;

        match attributes {
            Value::String(version) => Ok(Dependency::Package {
                name: name.to_owned(),
                version: version.to_owned(),
            }),
            Value::Table(package_desc) => {
                if package_desc.contains_key("git") {
                    let Some(repo) = package_desc["git"].as_str().map(|repo| repo.to_string())
                    else {
                        return Err(format!("Dependency \"{}\" git is not a string", name));
                    };
                    let version = package_desc
                        .get("version")
                        .map(|version| {
                            version
                                .as_str()
                                .map(|version| version.to_string())
                                .ok_or(format!("Dependency \"{}\" version is not a string", name))
                        })
                        .transpose()?;
                    let revision = package_desc
                        .get("revision")
                        .map(|revision| {
                            revision
                                .as_str()
                                .map(|revision| revision.to_string())
                                .ok_or(format!("Dependency \"{}\" is not a string", name))
                        })
                        .transpose()?;

                    Ok(Dependency::Git {
                        name: name.to_owned(),
                        repo,
                        version,
                        revision,
                    })
                } else if package_desc.contains_key("path") {
                    let Some(path) = package_desc["path"].as_str().map(|path| path.to_string())
                    else {
                        return Err(format!("Dependency \"{}\" path is not a string", name));
                    };
                    workdir.push(path);

                    Ok(Dependency::Local {
                        name: name.to_owned(),
                        path: workdir,
                    })
                } else {
                    Err(format!(
                        "Dependency \"{}\" must have a \"git\" or a \"path\" attribute",
                        name
                    ))
                }
            }
            _ => Err(format!(
                "Dependency \"{}\" has a invalid format. It must be a string or a table",
                name
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_from_folder() {
        let package_path = PathBuf::from("res/blinky");
        let manifest = Manifest::from_folder(&package_path).unwrap();
        let mut rand_path = std::env::current_dir().unwrap();
        rand_path.push("../rand");

        let mut dependencies = vec![
            Dependency::Package {
                name: "zephyr".to_string(),
                version: "3.5.0".to_string(),
            },
            Dependency::Local {
                name: "rand".to_string(),
                path: rand_path,
            },
            Dependency::Git {
                name: "backpack".to_string(),
                repo: "https://github.com/matheuswhite/backpack.git".to_string(),
                version: Some("1.0.0".to_string()),
                revision: None,
            },
        ];
        dependencies.sort_by_key(|dep| dep.name());

        assert_eq!(
            manifest,
            Manifest {
                name: "blinky".to_string(),
                version: "0.1.0".to_string(),
                edition: Edition::Edition2025_1,
                package_type: PackageType::Binary,
                dependencies,
                src: vec!["src/*.c".to_string()],
                include: vec!["src/".to_string()],
            }
        );
    }

    #[test]
    fn test_manifest_from_file() {
        let package_path = PathBuf::from("res/blinky/Tailor.toml");
        let manifest = Manifest::from_file(&package_path).unwrap();
        let mut rand_path = std::env::current_dir().unwrap();
        rand_path.push("../rand");

        let mut dependencies = vec![
            Dependency::Package {
                name: "zephyr".to_string(),
                version: "3.5.0".to_string(),
            },
            Dependency::Local {
                name: "rand".to_string(),
                path: rand_path,
            },
            Dependency::Git {
                name: "backpack".to_string(),
                repo: "https://github.com/matheuswhite/backpack.git".to_string(),
                version: Some("1.0.0".to_string()),
                revision: None,
            },
        ];
        dependencies.sort_by_key(|dep| dep.name());

        assert_eq!(
            manifest,
            Manifest {
                name: "blinky".to_string(),
                version: "0.1.0".to_string(),
                edition: Edition::Edition2025_1,
                package_type: PackageType::Binary,
                dependencies,
                src: vec!["src/*.c".to_string()],
                include: vec!["src/".to_string()],
            }
        );
    }
}
