use std::borrow::Borrow;
use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::Deserialize;

#[derive(Default, Debug, Deserialize)]
pub struct Config {
	#[serde(default)]
	versions: Vec<Version>,
}

#[derive(Debug, Deserialize)]
pub struct Version {
	version: String,
	#[serde(with = "date")]
	expiration: DateTime<Utc>,
}

mod date {
	use std::ops::Add;

	use chrono::{DateTime, Duration, TimeZone, Utc};
	use serde::{Deserialize, Deserializer};

	const FORMAT: &str = "%Y-%m-%d %H:%M";

	pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
	where
		D: Deserializer<'de>,
	{
		let s = String::deserialize(deserializer)?;
		if s.trim() == "never" {
			Ok(Utc::now().add(Duration::days(365 * 50)))
		} else {
			Utc.datetime_from_str(&s, FORMAT).map_err(|_| {
				serde::de::Error::custom(format!(
					"Invalid date format \"{}\", must be \
				{}",
					s, FORMAT
				))
			})
		}
	}
}

impl Config {
	pub fn new<T>(content: T) -> serde_yaml::Result<Self>
	where
		T: Borrow<str>,
	{
		let content = content.borrow();
		if content.trim().is_empty() {
			Ok(Default::default())
		} else {
			serde_yaml::from_str::<Config>(content)
		}
	}

	pub fn to_versions(self) -> HashMap<String, DateTime<Utc>> {
		self.versions
			.into_iter()
			.map(|v| (v.version, v.expiration))
			.collect()
	}
}

#[cfg(test)]
mod test {
	use chrono::{Datelike, Timelike};

	use crate::config::Config;

	#[test]
	fn should_parse_empty() {
		let content = r#""#;
		let config = Config::new(content);
		assert!(config.unwrap().versions.is_empty())
	}
	#[test]
	fn should_parse_empty_versions() {
		let content = r#"
			versions:
		"#;
		let config = Config::new(content);
		assert!(config.unwrap().versions.is_empty())
	}

	#[test]
	fn should_parse_never_expiration() {
		let content = r#"
			versions:
				- version: 1.0.0
				  expiration: never
		"#
		.replace('\t', " ");
		let config = Config::new(content);
		assert_eq!(config.unwrap().versions.len(), 1)
	}

	#[test]
	fn should_parse_invalid_expiration() {
		let content = r#"
			versions:
				- version: 1.0.0
				  expiration: 223234
		"#
		.replace('\t', " ");
		let config = Config::new(content);
		assert!(config.is_err())
	}

	#[test]
	fn should_parse_expiration() {
		let content = r#"
			versions:
				- version: 1.0.0
				  expiration: 2021-12-10 15:17
		"#
		.replace('\t', " ");
		let config = Config::new(content).unwrap();
		let versions = config.to_versions();
		let expiration = versions.get("1.0.0").unwrap();
		assert_eq!(expiration.year(), 2021);
		assert_eq!(expiration.month(), 12);
		assert_eq!(expiration.day(), 10);
		assert_eq!(expiration.hour(), 15);
		assert_eq!(expiration.minute(), 17);
	}
}
