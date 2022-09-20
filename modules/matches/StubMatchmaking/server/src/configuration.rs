use std::{
	collections::HashMap,
	fs::File,
	io::{self, BufReader, Read},
	path::PathBuf,
};

use serde::Deserialize;

use crate::service::Rules;

#[derive(Debug)]
pub enum Error {
	DeserializationError(serde_yaml::Error),
	IOError(io::Error),
}

#[derive(Deserialize, serde::Serialize)]
pub struct YamlConfig {
	rulemap: Vec<TemplateRuleItem>,
}

impl YamlConfig {
	pub fn rulemap(self) -> HashMap<String, Rules> {
		self.rulemap.into_iter().map(|item| (item.template, item.rules)).collect()
	}

	pub fn from_file(path: PathBuf) -> Result<Self, Error> {
		let file = File::open(path).map_err(|e| Error::IOError(e))?;
		let reader = BufReader::new(file);
		let rulemap = Self::from_reader(reader)?;

		Ok(rulemap)
	}

	fn from_reader(reader: impl Read) -> Result<Self, Error> {
		serde_yaml::from_reader(reader).map_err(|e| Error::DeserializationError(e))
	}
}

#[derive(Deserialize, serde::Serialize)]
struct TemplateRuleItem {
	template: String,
	rules: Rules,
}

#[cfg(test)]
mod test {
	use std::io::BufReader;

	use crate::{configuration::YamlConfig, service::Rules};

	#[test]
	fn test_deserialize_rulemap_works() {
		let config = "
            rulemap:
            - template: Hello
              rules:
                max_user_count: 4
        ";

		let reader = BufReader::new(config.as_bytes());
		let config = YamlConfig::from_reader(reader).unwrap();

		assert_eq!(*config.rulemap().get("Hello").unwrap(), Rules { max_user_count: 4 });
	}
}
