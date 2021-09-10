use std::path::PathBuf;

use crate::service::room::group::GroupAlias;
use crate::service::room::PrefabAlias;

#[derive(Debug)]
pub enum Error {
	PrefabNotFound(PathBuf, PrefabAlias),
	GroupNotFound(PathBuf, GroupAlias),

	PrefabFieldNotExists(PathBuf, String),
	ObjectFieldExists(PathBuf, u32),

	Io(std::io::Error),
	Yaml(serde_yaml::Error),
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
			Error::GroupNotFound(path, group) => {
				write!(f, "{}: Group {} not found", path.display(), group)
			}
			Error::ObjectFieldExists(path, id) => {
				write!(f, "{}: Field {} exists", path.display(), id)
			}
			Error::PrefabFieldNotExists(path, name) => {
				write!(f, "{}: Field {} not found in prefab", path.display(), name)
			}
			Error::Io(err) => write!(f, "IO: {:?}", err),
			Error::Yaml(err) => write!(
				f,
				"Wrong file format {:?}: {:?}",
				err.location().map(|loc| (loc.line(), loc.column())),
				err
			),
		}
	}
}
