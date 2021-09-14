use std::fmt::Formatter;
use std::path::PathBuf;

use crate::service::room::group::GroupAlias;
use crate::service::room::PrefabAlias;

#[derive(Debug)]
pub enum Error {
	PrefabNotFound(PathBuf, PrefabAlias),
	GroupNotFound(GroupAlias),
	GroupParseError(serde_yaml::Error),
	PrefabFieldNotExists(PathBuf, String),
	ObjectFieldExists(PathBuf, u32),
	Io(std::io::Error),
	Yaml(serde_yaml::Error),
	GroupFileNotFound(),
}

impl From<std::io::Error> for Error {
	fn from(err: std::io::Error) -> Self {
		Self::Io(err)
	}
}

impl From<serde_yaml::Error> for Error {
	fn from(err: serde_yaml::Error) -> Self {
		Self::Yaml(err)
	}
}

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Error::PrefabNotFound(path, prefab) => {
				write!(f, "{}: Prefab {} not found", path.display(), prefab.display(),)
			}
			Error::GroupNotFound(group) => {
				write!(f, "Group {} not found", group)
			}
			Error::ObjectFieldExists(path, id) => {
				write!(f, "{}: Field {} exists", path.display(), id)
			}
			Error::PrefabFieldNotExists(path, name) => {
				write!(f, "{}: Field {} not found in prefab", path.display(), name)
			}
			Error::Io(err) => write!(f, "IO: {:?}", err),
			Error::Yaml(err) => write_yaml_error(f, err),
			Error::GroupParseError(err) => write_yaml_error(f, err),
			Error::GroupFileNotFound() => {
				write!(f, "File groups.yaml not found")
			}
		}
	}
}

fn write_yaml_error(f: &mut Formatter, err: &serde_yaml::Error) -> std::fmt::Result {
	write!(
		f,
		"Wrong file format {:?}: {:?}",
		err.location().map(|loc| (loc.line(), loc.column())),
		err
	)
}
