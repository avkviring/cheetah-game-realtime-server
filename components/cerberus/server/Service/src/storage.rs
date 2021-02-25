pub use redis::{AsyncCommands, Commands, ConnectionLike, ErrorKind, RedisError, RedisResult};

///
/// Хранение данных по токенам обновления в Redis
/// для пары user + device_id хранится номер выданного токена
///
/// hmap[user_id][device_id] = token_seq
///
pub struct RedisRefreshTokenStorage {
    host: String,
    port: u16,
    ///
    /// Время жизни данных для пользователя
    ///
    time_of_life_in_sec: i64,
}

impl RedisRefreshTokenStorage {
    ///
    /// Максимальный размер device_id для исключения атаки на размер базы данных в Redis
    ///
    const DEVICE_ID_MAX_LEN: usize = 16;

    ///
    /// Количество устройств пользователя, для исключения атак на размер базы данных в Redis
    ///
    const COUNT_DEVICES_PER_USER: usize = 32;

    pub fn new(host: String, port: u16, time_of_life_in_sec: i64) -> Result<Self, String> {
        Result::Ok(Self {
            host,
            port,
            time_of_life_in_sec,
        })
    }

    fn make_key(player: u64) -> String {
        format!("r:{}", player)
    }

    fn normalize_device_id(device_id: &str) -> String {
        if device_id.len() > RedisRefreshTokenStorage::DEVICE_ID_MAX_LEN {
            device_id[0..RedisRefreshTokenStorage::DEVICE_ID_MAX_LEN].to_string()
        } else {
            device_id.to_owned()
        }
    }

    pub(crate) async fn new_version(
        &self,
        player: u64,
        device_id: &str,
    ) -> Result<u64, RedisError> {
        let key = RedisRefreshTokenStorage::make_key(player);
        let device_id = RedisRefreshTokenStorage::normalize_device_id(device_id);

        let client = redis::Client::open(format!("redis://{}:{}", self.host, self.port))?;
        let mut connection = client.get_async_connection().await?;

        let len: usize = connection.hlen(&key).await?;

        if len > RedisRefreshTokenStorage::COUNT_DEVICES_PER_USER {
            connection.del::<&String, usize>(&key).await?;
        }

        let result = connection.hincr(&key, device_id, 1 as u64).await?;

        connection
            .expire(&key, self.time_of_life_in_sec as usize)
            .await?;

        Result::Ok(result)
    }

    pub(crate) async fn get_version(
        &self,
        player: u64,
        device_id: &str,
    ) -> Result<u64, RedisError> {
        let key = RedisRefreshTokenStorage::make_key(player);
        let device_id = RedisRefreshTokenStorage::normalize_device_id(device_id);
        let client = redis::Client::open(format!("redis://{}:{}", self.host, self.port))?;
        let mut connection = client.get_async_connection().await?;
        connection
            .expire(&key, self.time_of_life_in_sec as usize)
            .await?;
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

    use crate::storage::RedisRefreshTokenStorage;

    #[tokio::test]
    async fn should_increment_version() {
        let (_node, storage) = stub_storage();

        let player = 123;
        let device = "device";
        let version_1 = storage.new_version(player, &device);
        let version_2 = storage.new_version(player, &device);
        let (version_1, version_2) = futures::join!(version_1, version_2);
        assert_ne!(version_1.unwrap(), version_2.unwrap());
    }

    #[tokio::test]
    async fn should_get_version() {
        let (_node, storage) = stub_storage();

        let player = 123;
        let device = "device".to_owned();
        let version_1 = storage.new_version(player, &device).await;
        let version_2 = storage.get_version(player, &device).await;
        assert_eq!(version_1.unwrap(), version_2.unwrap())
    }

    #[tokio::test]
    async fn should_get_unset_version() {
        let (_node, storage) = stub_storage();
        let player = 123;
        let device = "device".to_owned();
        let version = storage.get_version(player, &device).await;
        assert_eq!(version.unwrap(), 0)
    }

    #[tokio::test]
    async fn should_clear_after_timeout() {
        let (_node, storage) = stub_storage();
        let player = 123;
        let device = "device".to_owned();
        storage.new_version(player, &device).await.unwrap();
        thread::sleep(Duration::from_secs(2));
        let version = storage.get_version(player, &device).await;
        assert_eq!(version.unwrap(), 0)
    }

    #[tokio::test]
    async fn should_clear_if_so_much_user_id() {
        let (_node, storage) = stub_storage();
        let player = 123;
        let device = "device".to_owned();

        storage.new_version(player, &device).await.unwrap();
        for i in 0..RedisRefreshTokenStorage::COUNT_DEVICES_PER_USER + 1 {
            let device_i = format!("device-{}", i);
            storage.new_version(player, &device_i).await.unwrap();
        }

        let version = storage.get_version(player, &device).await;
        assert_eq!(version.unwrap(), 0)
    }

    #[tokio::test]
    async fn should_truncate_device_id() {
        let (_node, storage) = stub_storage();
        let player = 123.to_owned();
        let device_long_name = "012345678901234567890123456789".to_owned();
        let device_short_name =
            device_long_name[0..RedisRefreshTokenStorage::DEVICE_ID_MAX_LEN].to_owned();

        storage
            .new_version(player, &device_long_name)
            .await
            .unwrap();

        let version = storage.get_version(player, &device_short_name).await;
        assert_eq!(version.unwrap(), 1)
    }

    lazy_static::lazy_static! {
        static ref CLI: clients::Cli = Default::default();
    }

    pub fn stub_storage<'a>() -> (Container<'a, Cli, Redis>, RedisRefreshTokenStorage) {
        let node = (*CLI).run(images::redis::Redis::default());
        let port = node.get_host_port(6379).unwrap();
        (
            node,
            RedisRefreshTokenStorage::new("127.0.0.1".to_owned(), port, 1).unwrap(),
        )
    }
}
