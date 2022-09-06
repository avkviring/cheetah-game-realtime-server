use crate::proto::matches::registry::internal::RelayState;
use crate::registry::relay_addrs::Addrs;
use async_trait::async_trait;
use futures::future;
use redis::aio::MultiplexedConnection;
use redis::{AsyncCommands, RedisError};
use serde_json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageError {
	#[error("No available relay found")]
	NoRelayFound,
	#[error(transparent)]
	RedisError(#[from] RedisError),
	#[error(transparent)]
	MalformedValue(#[from] serde_json::Error),
}

#[async_trait]
pub trait Storage: Send + Sync {
	async fn get_random_relay_addr(&self) -> Result<Addrs, StorageError>;
	async fn update_status(&self, addrs: &Addrs, state: RelayState) -> Result<(), StorageError>;
	async fn remove_relay(&self, addrs: &Addrs) -> Result<(), StorageError>;
}

const REDIS_SET_KEY_READY: &str = "registry:relay-addrs:ready";
const REDIS_SET_KEY_ALLOCATED: &str = "registry:relay-addrs:allocated";

#[derive(Clone)]
pub struct RedisStorage {
	conn: MultiplexedConnection,
}

#[async_trait]
impl Storage for RedisStorage {
	/// Получить адрес случайного Relay сервера
	/// Сначала ищется Allocated сервер. Если его нет - берется Ready
	/// Если нет Ready - значит что-то не так с Agones FleetAutoscaler и возвращается ошибка NoRelayFound
	async fn get_random_relay_addr(&self) -> Result<Addrs, StorageError> {
		match self.srandmember(REDIS_SET_KEY_ALLOCATED).await {
			Ok(addrs) => Ok(addrs),
			Err(e) => match e {
				StorageError::NoRelayFound => {
					tracing::info!("no relay found in allocated set. getting from ready set");
					self.srandmember(REDIS_SET_KEY_READY).await
				}
				_ => Err(e),
			},
		}
	}

	/// Добавить или обновить Relay в хранилище
	async fn update_status(&self, addrs: &Addrs, state: RelayState) -> Result<(), StorageError> {
		match state {
			RelayState::Ready => {
				tracing::info!("adding relay to ready set {:?}", addrs);
				self.ensure_addrs_in_sets(addrs, REDIS_SET_KEY_ALLOCATED, REDIS_SET_KEY_READY).await
			}
			RelayState::Allocated => {
				tracing::info!("adding relay to allocated set {:?}", addrs);
				self.ensure_addrs_in_sets(addrs, REDIS_SET_KEY_READY, REDIS_SET_KEY_ALLOCATED).await
			}
			RelayState::NotReady => self.remove_relay(addrs).await,
		}
	}

	/// Удалить Relay из хранилища
	async fn remove_relay(&self, addrs: &Addrs) -> Result<(), StorageError> {
		tracing::info!("removing relay {:?}", addrs);
		future::try_join(self.srem(addrs, REDIS_SET_KEY_ALLOCATED), self.srem(addrs, REDIS_SET_KEY_READY))
			.await
			.map(|_| ())
	}
}

impl RedisStorage {
	/// Создать новый RedisStorage
	/// RedisStorage использует multiplexed соединение к Redis
	/// RedisStorage можно клонировать
	pub async fn new(dsn: &str) -> Result<Self, StorageError> {
		tracing::info!("connecting to redis: {:?}", dsn);
		let client = redis::Client::open(dsn)?;
		client
			.get_multiplexed_tokio_connection()
			.await
			.map(|conn| Self { conn })
			.map_err(StorageError::from)
	}

	async fn ensure_addrs_in_sets(&self, addrs: &Addrs, remove_from: &str, add_to: &str) -> Result<(), StorageError> {
		future::try_join(self.srem(addrs, remove_from), self.sadd(addrs, add_to))
			.await
			.map(|_| ())
	}

	async fn sadd(&self, addrs: &Addrs, key: &str) -> Result<(), StorageError> {
		self.conn
			.clone()
			.sadd(key, serde_json::to_vec(addrs).map_err(StorageError::from)?)
			.await
			.map_err(StorageError::from)
	}

	async fn srem(&self, addrs: &Addrs, key: &str) -> Result<(), StorageError> {
		self.conn
			.clone()
			.srem(key, serde_json::to_vec(addrs).map_err(StorageError::from)?)
			.await
			.map_err(StorageError::from)
	}

	async fn srandmember(&self, key: &str) -> Result<Addrs, StorageError> {
		let res: Vec<u8> = self.conn.clone().srandmember(key).await.map_err(StorageError::from)?;
		if res.is_empty() {
			return Err(StorageError::NoRelayFound);
		}

		let addrs: Addrs = serde_json::from_slice(&*res).map_err(StorageError::from)?;
		Ok(addrs)
	}
}

#[cfg(test)]
pub mod tests {
	use crate::proto::matches::registry::internal::RelayState;
	use crate::registry::relay_addrs::Addrs;
	use crate::registry::storage::{RedisStorage, Storage, StorageError};
	use std::net::SocketAddr;
	use std::str::FromStr;
	use testcontainers::clients::Cli;
	use testcontainers::images::redis::Redis;
	use testcontainers::{Container, Docker};

	#[tokio::test]
	async fn should_return_err_on_empty_set() {
		let (_node, storage) = stub_storage().await;
		let res = storage.get_random_relay_addr().await;
		assert!(matches!(res, Err(StorageError::NoRelayFound)))
	}

	#[tokio::test]
	async fn should_return_err_on_not_ready() {
		let (_node, storage) = stub_storage().await;
		let want = Addrs {
			game: SocketAddr::from_str("127.0.0.1:80").unwrap(),
			grpc_internal: SocketAddr::from_str("127.0.0.2:90").unwrap(),
		};
		storage.update_status(&want, RelayState::Ready).await.unwrap();
		storage.update_status(&want, RelayState::NotReady).await.unwrap();
		let res = storage.get_random_relay_addr().await;

		assert!(matches!(res, Err(StorageError::NoRelayFound)))
	}

	#[tokio::test]
	async fn should_return_addr() {
		let (_node, storage) = stub_storage().await;
		let want = Addrs {
			game: SocketAddr::from_str("127.0.0.1:80").unwrap(),
			grpc_internal: SocketAddr::from_str("127.0.0.2:90").unwrap(),
		};

		storage.update_status(&want, RelayState::Ready).await.unwrap();
		let got = storage.get_random_relay_addr().await.unwrap();

		assert_eq!(got, want)
	}

	lazy_static::lazy_static! {
		static ref CLI: Cli = Default::default();
	}

	async fn stub_storage<'a>() -> (Container<'a, Cli, Redis>, RedisStorage) {
		let node = (*CLI).run(Redis::default());
		let port = node.get_host_port(6379).unwrap();
		(node, RedisStorage::new(&format!("redis://127.0.0.1:{}", port)).await.unwrap())
	}
}
