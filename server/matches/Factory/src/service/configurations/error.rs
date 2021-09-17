use std::fmt::Formatter;
use std::path::PathBuf;

pub enum Error {
	Io(std::io::Error),
	Yaml {
		global_root: PathBuf,
		file: PathBuf,
		e: serde_yaml::Error,
	},
	GroupFileNotFound,
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
		}
	}
}

fn write_yaml_error(f: &mut Formatter, file: &PathBuf, err: &serde_yaml::Error) -> std::fmt::Result {
	write!(
		f,
		"Error in file {}. Wrong format {:?}: {:?}",
		file.display().to_string(),
		err.location().map(|loc| (loc.line(), loc.column())),
		err
	)
}
