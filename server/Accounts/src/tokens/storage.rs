use redis::aio::{Connection, MultiplexedConnection};
pub use redis::{AsyncCommands, Commands, ConnectionLike, ErrorKind, RedisError, RedisResult};

use crate::users::UserId;

///
/// Хранение данных по токенам обновления в Redis.
///
/// hmap[user_id][device_id] = token_seq
///
#[derive(Clone)]
pub struct TokenStorage {
	///
	/// Время жизни данных для пользователя
	///
	time_of_life_in_sec: u64,
	redis: MultiplexedConnection,
}

impl TokenStorage {
	///
	/// Максимальный размер device_id для исключения атаки на размер базы данных в Redis
	///
	const DEVICE_ID_MAX_LEN: usize = 16;

	///
	/// Количество устройств пользователя, для исключения атак на размер базы данных в Redis
	///
	const COUNT_DEVICES_PER_USER: usize = 32;

	pub async fn new(host: &str, port: u16, auth: Option<String>, time_of_life_in_sec: u64) -> Result<Self, String> {
		let url = match auth {
			Option::Some(ref password) => {
				format!("redis://:{}@{}:{}", password, host, port)
			}
			Option::None => {
				format!("redis://{}:{}", host, port)
			}
		};

		let redis = redis::Client::open(url)
			.unwrap()
			.get_multiplexed_tokio_connection()
			.await
			.unwrap();
		redis
			.clone()
			.set::<String, String, ()>("test".to_string(), "value".to_string())
			.await
			.unwrap();

		Ok(Self {
			time_of_life_in_sec,
			redis,
		})
	}

	fn make_key(user_id: UserId) -> String {
		format!("r:{}", i64::from(user_id))
	}

	fn normalize_device_id(device_id: &str) -> String {
		if device_id.len() > TokenStorage::DEVICE_ID_MAX_LEN {
			device_id[0..TokenStorage::DEVICE_ID_MAX_LEN].to_string()
		} else {
			device_id.to_owned()
		}
	}

	pub(crate) async fn new_version(&self, user: UserId, device_id: &str) -> Result<u64, RedisError> {
		let key = TokenStorage::make_key(user);
		let device_id = TokenStorage::normalize_device_id(device_id);
		let mut connection = self.redis.clone();
		let len: usize = connection.hlen(&key).await?;
		if len > TokenStorage::COUNT_DEVICES_PER_USER {
			connection.del::<&String, usize>(&key).await?;
		}
		let result = connection.hincr(&key, device_id, 1_u64).await?;

		connection.expire(&key, self.time_of_life_in_sec as usize).await?;

		Result::Ok(result)
	}

	pub(crate) async fn get_version(&self, user_id: UserId, device_id: &str) -> Result<u64, RedisError> {
		let key = TokenStorage::make_key(user_id);
		let device_id = TokenStorage::normalize_device_id(device_id);
		let mut connection = self.redis.clone();
		connection.expire(&key, self.time_of_life_in_sec as usize).await?;
		let result: Result<Option<u64>, RedisError> = connection.hget(key, device_id).await;
		result.map(|v| v.unwrap_or(0))
	}
}

#[cfg(test)]
pub mod tests {
	use std::thread;
	use std::time::Duration;

	use testcontainers::clients::Cli;
	use testcontainers::images::redis::Redis;
	use testcontainers::{clients, images, Container, Docker};

	use crate::tokens::storage::TokenStorage;
	use crate::users::UserId;

	#[tokio::test]
	async fn should_increment_version() {
		let (_node, storage) = stub_storage().await;

		let user_id = UserId::from(123u64);
		let device = "device";
		let version_1 = storage.new_version(user_id, device);
		let version_2 = storage.new_version(user_id, device);
		let (version_1, version_2) = futures::join!(version_1, version_2);
		assert_ne!(version_1.unwrap(), version_2.unwrap());
	}

	#[tokio::test]
	async fn should_get_version() {
		let (_node, storage) = stub_storage().await;

		let user_id = UserId::from(123u64);
		let device = "device".to_owned();
		let version_1 = storage.new_version(user_id, &device).await;
		let version_2 = storage.get_version(user_id, &device).await;
		assert_eq!(version_1.unwrap(), version_2.unwrap())
	}

	#[tokio::test]
	async fn should_get_unset_version() {
		let (_node, storage) = stub_storage().await;
		let user_id = UserId::from(123u64);
		let device = "device".to_owned();
		let version = storage.get_version(user_id, &device).await;
		assert_eq!(version.unwrap(), 0)
	}

	#[tokio::test]
	async fn should_clear_after_timeout() {
		let (_node, storage) = stub_storage().await;
		let user_id = UserId::from(123u64);
		let device = "device".to_owned();
		storage.new_version(user_id, &device).await.unwrap();
		thread::sleep(Duration::from_secs(2));
		let version = storage.get_version(user_id, &device).await;
		assert_eq!(version.unwrap(), 0)
	}

	#[tokio::test]
	async fn should_clear_if_so_much_user_id() {
		let (_node, storage) = stub_storage().await;
		let user_id = UserId::from(123u64);
		let device = "device".to_owned();

		storage.new_version(user_id, &device).await.unwrap();
		for i in 0..TokenStorage::COUNT_DEVICES_PER_USER + 1 {
			let device_i = format!("device-{}", i);
			storage.new_version(user_id, &device_i).await.unwrap();
		}

		let version = storage.get_version(user_id, &device).await;
		assert_eq!(version.unwrap(), 0)
	}

	#[tokio::test]
	async fn should_truncate_device_id() {
		let (_node, storage) = stub_storage().await;
		let user_id = UserId::from(123u64);
		let device_long_name = "012345678901234567890123456789".to_owned();
		let device_short_name = device_long_name[0..TokenStorage::DEVICE_ID_MAX_LEN].to_owned();

		storage.new_version(user_id, &device_long_name).await.unwrap();

		let version = storage.get_version(user_id, &device_short_name).await;
		assert_eq!(version.unwrap(), 1)
	}

	lazy_static::lazy_static! {
		static ref CLI: clients::Cli = Default::default();
	}

	pub async fn stub_storage<'a>() -> (Container<'a, Cli, Redis>, TokenStorage) {
		let node = (*CLI).run(images::redis::Redis::default());
		let port = node.get_host_port(6379).unwrap();
		(node, TokenStorage::new("127.0.0.1", port, Option::None, 1).await.unwrap())
	}
}
