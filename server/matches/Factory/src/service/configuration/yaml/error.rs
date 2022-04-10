use std::fmt::Formatter;
use std::path::{Path, PathBuf};

use crate::service::configuration::yaml::structures::{FieldName, TemplateName};

pub enum Error {
	Io(std::io::Error),
	Yaml {
		global_root: PathBuf,
		file: PathBuf,
		e: serde_yaml::Error,
	},
	GroupFileNotFound,
	TemplateWithSameIdAlreadyExists {
		id: u32,
		exist: TemplateName,
		current: TemplateName,
	},
	FieldWithSameIdAlreadyExists {
		id: u16,
		exist: FieldName,
		current: FieldName,
	},
	NameAlreadyExists {
		name: String,
		file: PathBuf,
	},
	CannotCreateDefault {
		name: String,
		file: PathBuf,
	},
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
	fn from(err: std::io::Error) -> Self {
		Self::Io(err)
	}
}

impl std::fmt::Debug for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		std::fmt::Display::fmt(self, f)
	}
}

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Error::Io(err) => write!(f, "IO: {:?}", err),
			Error::GroupFileNotFound => write!(f, "File groups.yaml or groups.yml not found"),
			Error::Yaml { global_root, file, e } => {
				let local_file = file.clone().strip_prefix(global_root.as_path()).unwrap().to_path_buf();
				write_yaml_error(f, &local_file, e)
			}
			Error::TemplateWithSameIdAlreadyExists { id, exist, current } => {
				write!(f, "Templates {} and {} has same id {} ", exist, current, id)
			}
			Error::FieldWithSameIdAlreadyExists { id, exist, current } => {
				write!(f, "Fields {} and {} has same id {} ", exist, current, id)
			}
			Error::NameAlreadyExists { name, file } => {
				write!(f, "Name {} already exists in file {:?}", name, file)
			}
			Error::CannotCreateDefault { name: _, file } => {
				write!(f, "Cannot create default struct for empty file {:?}", file)
			}
		}
	}
}

fn write_yaml_error(f: &mut Formatter, file: &Path, err: &serde_yaml::Error) -> std::fmt::Result {
	write!(
		f,
		"Error in file {}. Wrong format {:?}: {:?}",
		file.display().to_string(),
		err.location().map(|loc| (loc.line(), loc.column())),
		err
	)
}
