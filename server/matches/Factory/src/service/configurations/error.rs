use std::fmt::Formatter;

#[derive(Debug)]
pub enum Error {
	GroupParseError(serde_yaml::Error),
	Io(std::io::Error),
	Yaml(serde_yaml::Error),
	GroupFileNotFound,
}

impl std::error::Error for Error {}

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
			Error::Io(err) => write!(f, "IO: {:?}", err),
			Error::Yaml(err) => write_yaml_error(f, err),
			Error::GroupParseError(err) => write_yaml_error(f, err),
			Error::GroupFileNotFound => write!(f, "File groups.yaml or groups.yml not found"),
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
