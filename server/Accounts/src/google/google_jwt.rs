extern crate core;

use std::collections::HashMap;
use std::time::Instant;

use jsonwebtoken::errors::Error;
use jsonwebtoken::DecodingKey;
use jsonwebtoken::{Algorithm, Validation};
use serde::de::DeserializeOwned;
use serde::Deserialize;

use headers::Header;
use reqwest::header::{HeaderMap, CACHE_CONTROL};
use thiserror::Error;

///
/// Parser errors
///
#[derive(Error, Debug)]
pub enum ParserError {
	#[error("Wrong header.")]
	WrongHeader,
	#[error("Unknown kid.")]
	UnknownKid,
	#[error("Download public key error - {0}.")]
	KeyProvider(GoogleKeyProviderError),
	#[error("Wrong token format - {0}.")]
	WrongToken(jsonwebtoken::errors::Error),
}

///
/// Parse & Validate Google JWT token.
/// Use public key from http(s) server.
///
pub struct Parser {
	client_id: String,
	key_provider: tokio::sync::Mutex<GooglePublicKeyProvider>,
}

impl Parser {
	pub const GOOGLE_CERT_URL: &'static str = "https://www.googleapis.com/oauth2/v3/certs";

	pub fn new(client_id: &str) -> Self {
		Parser::new_with_custom_cert_url(client_id, Parser::GOOGLE_CERT_URL)
	}

	pub fn new_with_custom_cert_url(client_id: &str, public_key_url: &str) -> Self {
		Self {
			client_id: client_id.to_owned(),
			key_provider: tokio::sync::Mutex::new(GooglePublicKeyProvider::new(public_key_url)),
		}
	}

	///
	/// Parse and validate token.
	/// Download and cache public keys from http(s) server.
	/// Use expire time header for reload keys.
	///
	pub async fn parse<T: DeserializeOwned>(&self, token: &str) -> Result<T, ParserError> {
		let mut provider = self.key_provider.lock().await;
		match jsonwebtoken::decode_header(token) {
			Ok(header) => match header.kid {
				None => Result::Err(ParserError::UnknownKid),
				Some(kid) => match provider.get_key(kid.as_str()).await {
					Ok(key) => {
						let aud = vec![self.client_id.to_owned()];
						let mut validation = Validation::new(Algorithm::RS256);
						validation.set_audience(&aud);
						validation.set_issuer(&["https://accounts.google.com".to_string(), "accounts.google.com".to_string()]);
						validation.validate_exp = true;
						validation.validate_nbf = false;
						let result = jsonwebtoken::decode::<T>(token, &key, &validation);
						match result {
							Result::Ok(token_data) => Result::Ok(token_data.claims),
							Result::Err(error) => Result::Err(ParserError::WrongToken(error)),
						}
					}
					Err(e) => {
						let error = ParserError::KeyProvider(e);
						Result::Err(error)
					}
				},
			},
			Err(_) => Result::Err(ParserError::WrongHeader),
		}
	}
}

#[derive(Deserialize, Clone)]
pub struct GoogleKeys {
	keys: Vec<GoogleKey>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct GoogleKey {
	kid: String,
	n: String,
	e: String,
}

#[derive(Error, Debug)]
pub enum GoogleKeyProviderError {
	#[error("key not found")]
	KeyNotFound,
	#[error("network error {0}")]
	FetchError(String),
	#[error("parse error {0}")]
	ParseError(String),
	#[error("create key error {0}")]
	CreateKeyError(Error),
}

#[derive(Debug)]
pub struct GooglePublicKeyProvider {
	url: String,
	keys: HashMap<String, GoogleKey>,
	expiration_time: Option<Instant>,
}

impl GooglePublicKeyProvider {
	pub fn new(public_key_url: &str) -> Self {
		Self {
			url: public_key_url.to_owned(),
			keys: Default::default(),
			expiration_time: None,
		}
	}

	pub async fn reload(&mut self) -> Result<(), GoogleKeyProviderError> {
		match reqwest::get(&self.url).await {
			Ok(r) => {
				let expiration_time = GooglePublicKeyProvider::parse_expiration_time(&r.headers());
				match r.json::<GoogleKeys>().await {
					Ok(google_keys) => {
						self.keys.clear();
						for key in google_keys.keys.into_iter() {
							self.keys.insert(key.kid.clone(), key);
						}
						self.expiration_time = expiration_time;
						Result::Ok(())
					}
					Err(e) => Result::Err(GoogleKeyProviderError::ParseError(format!("{:?}", e))),
				}
			}
			Err(e) => Result::Err(GoogleKeyProviderError::FetchError(format!("{:?}", e))),
		}
	}

	fn parse_expiration_time(header_map: &HeaderMap) -> Option<Instant> {
		match headers::CacheControl::decode(&mut header_map.get_all(CACHE_CONTROL).iter()) {
			Ok(header) => match header.max_age() {
				None => None,
				Some(max_age) => Some(Instant::now() + max_age),
			},
			Err(_) => None,
		}
	}

	pub fn is_expire(&self) -> bool {
		if let Some(expire) = self.expiration_time {
			Instant::now() > expire
		} else {
			false
		}
	}

	pub async fn get_key(&mut self, kid: &str) -> Result<DecodingKey, GoogleKeyProviderError> {
		if self.expiration_time.is_none() || self.is_expire() {
			self.reload().await?
		}
		match self.keys.get(&kid.to_owned()) {
			None => Result::Err(GoogleKeyProviderError::KeyNotFound),
			Some(key) => DecodingKey::from_rsa_components(key.n.as_str(), key.e.as_str())
				.map_err(|e| GoogleKeyProviderError::CreateKeyError(e)),
		}
	}
}

#[cfg(test)]
mod tests {
	use std::time::Duration;

	use jsonwebtoken::errors::ErrorKind;

	use crate::google::google_jwt::{GoogleKeyProviderError, GooglePublicKeyProvider, ParserError};
	use crate::google::test_helper::{setup, TokenClaims};
	use httpmock::MockServer;

	#[tokio::test]
	async fn should_parse_keys() {
		let n = "3g46w4uRYBx8CXFauWh6c5yO4ax_VDu5y8ml_Jd4Gx711155PTdtLeRuwZOhJ6nRy8YvLFPXc_aXtHifnQsi9YuI_vo7LGG2v3CCxh6ndZBjIeFkxErMDg4ELt2DQ0PgJUQUAKCkl2_gkVV9vh3oxahv_BpIgv1kuYlyQQi5JWeF7zAIm0FaZ-LJT27NbsCugcZIDQg9sztTN18L3-P_kYwvAkKY2bGYNU19qLFM1gZkzccFEDZv3LzAz7qbdWkwCoK00TUUH8TNjqmK67bytYzgEgkfF9q9szEQ5TrRL0uFg9LxT3kSTLYqYOVaUIX3uaChwaa-bQvHuNmryu7i9w";
		let e = "AQAB";
		let kid = "some-kid";
		let resp = format!("{{\"keys\": [{{\"kty\": \"RSA\",\"use\": \"sig\",\"e\": \"{}\",\"n\": \"{}\",\"alg\": \"RS256\",\"kid\": \"{}\"}}]}}", e, n, kid);

		let server = MockServer::start();
		let _server_mock = server.mock(|when, then| {
			when.method(httpmock::Method::GET).path("/");

			then.status(200)
				.header("cache-control", "public, max-age=24920, must-revalidate, no-transform")
				.header("Content-Type", "application/json; charset=UTF-8")
				.body(resp);
		});
		let mut provider = GooglePublicKeyProvider::new(server.url("/").as_str());

		assert!(matches!(provider.get_key(kid).await, Result::Ok(_)));
		assert!(matches!(provider.get_key("missing-key").await, Result::Err(_)));
	}

	#[tokio::test]
	async fn should_expire_and_reload() {
		let server = MockServer::start();
		let n = "3g46w4uRYBx8CXFauWh6c5yO4ax_VDu5y8ml_Jd4Gx711155PTdtLeRuwZOhJ6nRy8YvLFPXc_aXtHifnQsi9YuI_vo7LGG2v3CCxh6ndZBjIeFkxErMDg4ELt2DQ0PgJUQUAKCkl2_gkVV9vh3oxahv_BpIgv1kuYlyQQi5JWeF7zAIm0FaZ-LJT27NbsCugcZIDQg9sztTN18L3-P_kYwvAkKY2bGYNU19qLFM1gZkzccFEDZv3LzAz7qbdWkwCoK00TUUH8TNjqmK67bytYzgEgkfF9q9szEQ5TrRL0uFg9LxT3kSTLYqYOVaUIX3uaChwaa-bQvHuNmryu7i9w";
		let e = "AQAB";
		let kid = "some-kid";
		let resp = format!("{{\"keys\": [{{\"kty\": \"RSA\",\"use\": \"sig\",\"e\": \"{}\",\"n\": \"{}\",\"alg\": \"RS256\",\"kid\": \"{}\"}}]}}", e, n, kid);

		let mut server_mock = server.mock(|when, then| {
			when.method(httpmock::Method::GET).path("/");
			then.status(200)
				.header("cache-control", "public, max-age=3, must-revalidate, no-transform")
				.header("Content-Type", "application/json; charset=UTF-8")
				.body("{\"keys\":[]}");
		});

		let mut provider = GooglePublicKeyProvider::new(server.url("/").as_str());
		let key_result = provider.get_key(kid).await;
		assert!(matches!(key_result, Result::Err(GoogleKeyProviderError::KeyNotFound)));

		server_mock.delete();
		let _server_mock = server.mock(|when, then| {
			when.method(httpmock::Method::GET).path("/");
			then.status(200)
				.header("cache-control", "public, max-age=3, must-revalidate, no-transform")
				.header("Content-Type", "application/json; charset=UTF-8")
				.body(resp);
		});

		std::thread::sleep(Duration::from_secs(4));
		let key_result = provider.get_key(kid).await;
		assert!(matches!(key_result, Result::Ok(_)));
	}

	#[tokio::test]
	async fn should_correct() {
		let claims = TokenClaims::new();
		let (token, parser, _server) = setup(&claims);
		let result = parser.parse::<TokenClaims>(token.as_str()).await;
		let result = result.unwrap();
		assert_eq!(result.email, claims.email);
	}

	#[tokio::test]
	async fn should_validate_exp() {
		let claims = TokenClaims::new_expired();
		let (token, validator, _server) = setup(&claims);
		let result = validator.parse::<TokenClaims>(token.as_str()).await;

		assert!(if let ParserError::WrongToken(error) = result.err().unwrap() {
			if let ErrorKind::ExpiredSignature = error.into_kind() {
				true
			} else {
				false
			}
		} else {
			false
		});
	}

	#[tokio::test]
	async fn should_validate_iss() {
		let mut claims = TokenClaims::new();
		claims.iss = "https://some.com".to_owned();
		let (token, validator, _server) = setup(&claims);
		let result = validator.parse::<TokenClaims>(token.as_str()).await;
		assert!(if let ParserError::WrongToken(error) = result.err().unwrap() {
			if let ErrorKind::InvalidIssuer = error.into_kind() {
				true
			} else {
				false
			}
		} else {
			false
		});
	}

	#[tokio::test]
	async fn should_validate_aud() {
		let mut claims = TokenClaims::new();
		claims.aud = "other-id".to_owned();
		let (token, validator, _server) = setup(&claims);
		let result = validator.parse::<TokenClaims>(token.as_str()).await;
		assert!(if let ParserError::WrongToken(error) = result.err().unwrap() {
			if let ErrorKind::InvalidAudience = error.into_kind() {
				true
			} else {
				false
			}
		} else {
			false
		});
	}
}
