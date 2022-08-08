use thiserror::Error;
use uuid::Uuid;
use ydb::{Bytes, TableClient, YdbOrCustomerError};
use ydb_steroids::error::is_unique_violation_error;
use ydb_steroids::{query, select, update};

use crate::cookie::Cookie;
use crate::users::user::User;

#[derive(Debug, Error)]
pub enum AttachError {
	#[error("UniqueViolation")]
	UniqueViolation,
	#[error("Other {0}")]
	Other(String),
}

pub struct CookieStorage {
	ydb_table_client: TableClient,
}

impl CookieStorage {
	pub fn new(ydb_table_client: TableClient) -> Self {
		Self { ydb_table_client }
	}

	pub async fn attach(&self, user: User) -> Result<Cookie, AttachError> {
		loop {
			let cookie = Cookie(Uuid::new_v4());
			match self.do_attach(user, &cookie).await {
				Ok(_) => return Ok(cookie),
				Err(AttachError::UniqueViolation) => continue,
				Err(err) => return Err(err),
			}
		}
	}

	async fn do_attach(&self, user: User, cookie: &Cookie) -> Result<(), AttachError> {
		let result = update!(
			self.ydb_table_client,
			query!(
				"insert into cookie_users (cookie,user) values($cookie, $user)", 
				cookie=>cookie,
				user=>user)
		)
		.await;

		result.map_err(|e| {
			if is_unique_violation_error(&e) {
				AttachError::UniqueViolation
			} else {
				AttachError::Other(format!("{:?}", e))
			}
		})
	}

	pub async fn find(&self, cookie: &Cookie) -> Result<Option<User>, YdbOrCustomerError> {
		let result: Vec<User> = select!(
			self.ydb_table_client,
			query!("select user from cookie_users where cookie=$cookie", cookie=>cookie),
			user => Bytes)
		.await?;
		Ok(result.into_iter().last())
	}
}

#[cfg(test)]
pub mod tests {
	use crate::cookie::storage::{AttachError, CookieStorage};
	use crate::cookie::Cookie;
	use crate::users::service::UserService;
	use crate::postgres::test::setup_ydb;

	#[tokio::test]
	pub async fn should_attach() {
		let (ydb_client, _instance) = setup_ydb().await;
		let user_service = UserService::new(ydb_client.table_client());
		let cookie_storage = CookieStorage::new(ydb_client.table_client());

		let user_a = user_service.create().await.unwrap();
		let user_b = user_service.create().await.unwrap();

		let cookie_a = cookie_storage.attach(user_a).await.unwrap();
		let cookie_b = cookie_storage.attach(user_b).await.unwrap();

		assert_eq!(
			cookie_storage.find(&cookie_a).await.unwrap().unwrap(),
			user_a
		);
		assert_eq!(
			cookie_storage.find(&cookie_b).await.unwrap().unwrap(),
			user_b
		);
	}

	#[tokio::test]
	pub async fn should_check_duplicate() {
		let (ydb_client, _instance) = setup_ydb().await;
		let user_service = UserService::new(ydb_client.table_client());
		let cookie_storage = CookieStorage::new(ydb_client.table_client());

		let user = user_service.create().await.unwrap();
		assert!(cookie_storage
			.do_attach(user, &Cookie::from(1u128))
			.await
			.is_ok());
		assert!(matches!(
			cookie_storage.do_attach(user, &Cookie::from(1u128)).await,
			Err(AttachError::UniqueViolation)
		));
	}
}
