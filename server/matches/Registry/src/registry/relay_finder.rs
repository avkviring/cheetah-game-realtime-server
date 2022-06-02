use crate::registry::relay_addrs::Addrs;
use crate::registry::relay_prober::RelayProber;
use crate::registry::storage::{Storage, StorageError};

pub struct RelayFinder {
	storage: Box<dyn Storage>,
	prober: Box<dyn RelayProber>,
}

impl RelayFinder {
	pub fn new(storage: Box<dyn Storage>, prober: Box<dyn RelayProber>) -> Self {
		RelayFinder { storage, prober }
	}

	/// Получить адрес доступного Relay из хранилища и проверяет что он доступен
	/// проверка необходима, т.к. в случае падения Registry и Relay одновременно, адрес Relay может остаться в хранилище.
	/// Если Relay недоступен, удаляем его из хранилища и пробуем получить новый адрес.
	pub async fn get_random_relay_addr(&self) -> Result<Addrs, StorageError> {
		loop {
			let addrs = self.storage.get_random_relay_addr().await?;
			tracing::debug!("probing relay {:?}", addrs);
			if let Err(e) = self.prober.probe(addrs.grpc_internal).await {
				tracing::warn!("relay {:?} probe failed: {:?}", addrs, e);
				self.storage.remove_relay(&addrs).await?;
				continue;
			}
			return Ok(addrs);
		}
	}
}

#[cfg(test)]
mod tests {
	use crate::proto::matches::registry::internal::RelayState;
	use crate::registry::relay_addrs::Addrs;
	use crate::registry::relay_finder::RelayFinder;
	use crate::registry::relay_prober::ReconnectProber;
	use crate::registry::storage::{RedisStorage, Storage, StorageError};
	use std::net::SocketAddr;
	use std::str::FromStr;
	use testcontainers::clients::Cli;
	use testcontainers::images::redis::Redis;
	use testcontainers::{Container, Docker};

	#[tokio::test]
	async fn test_no_relay_found() {
		let (_node, storage) = stub_storage().await;
		let want = Addrs {
			game: SocketAddr::from_str("127.0.0.1:80").unwrap(),
			grpc_internal: SocketAddr::from_str("127.0.0.2:90").unwrap(),
		};
		storage
			.update_status(&want, RelayState::Ready)
			.await
			.unwrap();

		let finder = RelayFinder {
			storage: Box::new(storage.clone()),
			prober: Box::new(ReconnectProber {}),
		};

		let res = finder.get_random_relay_addr().await;
		assert!(
			matches!(res, Err(StorageError::NoRelayFound)),
			"if relay is unreachable, error should be returned"
		);

		let res = storage.get_random_relay_addr().await;
		assert!(
			matches!(res, Err(StorageError::NoRelayFound)),
			"unreachable relay should be removed from storage"
		);
	}

	lazy_static::lazy_static! {
		static ref CLI: Cli = Default::default();
	}

	async fn stub_storage<'a>() -> (Container<'a, Cli, Redis>, RedisStorage) {
		let node = (*CLI).run(Redis::default());
		let port = node.get_host_port(6379).unwrap();
		(
			node,
			RedisStorage::new(&format!("redis://127.0.0.1:{}", port))
				.await
				.unwrap(),
		)
	}
}
