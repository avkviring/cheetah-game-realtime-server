use std::time::Duration;

use redis::{Client, Commands, ConnectionLike, RedisResult};

///
/// Хранение refresh токенов в persistent хранилище
///
pub trait Storage {
    ///
    /// Новая версия refresh токена
    ///
    fn new_version(&mut self, user_id: &String, device_id: &String) -> u64;

    ///
    /// Получить текущую версию refresh токена
    ///
    fn get_version(&mut self, user_id: &String, device_id: &String) -> u64;
}

///
/// Хранение данных по токенам обновления в Redis
/// для пары user + device_id хранится номер выданного токена
///
/// hmap[user_id][device_id] = token_seq
///
pub struct RedisRefreshTokenStorage {
    client: Client,
    ///
    /// Время жизни данных для пользователя
    ///
    time_of_life_in_sec: usize,
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

    pub fn new(host: String, port: u16, time_of_life_in_sec: usize) -> Result<Self, String> {
        let client = redis::Client::open(format!("redis://{}:{}", host, port))
            .map_err(|e| format!("{:?}", e))?;
        Result::Ok(Self {
            client,
            time_of_life_in_sec,
        })
    }
    fn set_expire(&mut self, key: &String) {
        self.client
            .expire::<&String, usize>(key, self.time_of_life_in_sec)
            .unwrap();
    }

    fn make_key(user_id: &String) -> String {
        format!("r:{}", user_id)
    }

    fn normalize_device_id(device_id: &String) -> String {
        if device_id.len() > RedisRefreshTokenStorage::DEVICE_ID_MAX_LEN {
            device_id[0..RedisRefreshTokenStorage::DEVICE_ID_MAX_LEN].to_string()
        } else {
            device_id.to_owned()
        }
    }
}

impl Storage for RedisRefreshTokenStorage {
    fn new_version(&mut self, user_id: &String, device_id: &String) -> u64 {
        let key = RedisRefreshTokenStorage::make_key(user_id);

        let len: usize = self.client.hlen(&key).unwrap();
        if len > RedisRefreshTokenStorage::COUNT_DEVICES_PER_USER {
            self.client.del::<&String, usize>(&key).unwrap();
        }

        let result: RedisResult<u64> = self.client.hincr(
            &key,
            RedisRefreshTokenStorage::normalize_device_id(device_id),
            1,
        );

        self.set_expire(&key);

        result.unwrap()
    }

    fn get_version(&mut self, user_id: &String, device_id: &String) -> u64 {
        let key = RedisRefreshTokenStorage::make_key(user_id);
        self.set_expire(&key);
        let t: RedisResult<u64> = self.client.hget(
            key,
            RedisRefreshTokenStorage::normalize_device_id(device_id),
        );
        t.unwrap_or(0)
    }
}

#[cfg(test)]
pub mod tests {
    use std::thread;
    use std::time::Duration;

    use testcontainers::{clients, images, Container, Docker};

    use crate::storage::{RedisRefreshTokenStorage, Storage};

    #[test]
    fn should_increment_version() {
        let cli = clients::Cli::default();
        let node = cli.run(images::redis::Redis::default());
        let mut storage = RedisRefreshTokenStorage::new(
            "127.0.0.1".to_owned(),
            node.get_host_port(6379).unwrap(),
            1,
        )
        .unwrap();

        let version_1 = storage.new_version(&"user-a".to_owned(), &"device-a".to_owned());
        let version_2 = storage.new_version(&"user-a".to_owned(), &"device-a".to_owned());
        assert_ne!(version_1, version_2)
    }

    #[test]
    fn should_get_version() {
        let cli = clients::Cli::default();
        let node = cli.run(images::redis::Redis::default());
        let mut storage = RedisRefreshTokenStorage::new(
            "127.0.0.1".to_owned(),
            node.get_host_port(6379).unwrap(),
            1,
        )
        .unwrap();

        let version_1 = storage.new_version(&"user-a".to_owned(), &"device-a".to_owned());
        let version_2 = storage.get_version(&"user-a".to_owned(), &"device-a".to_owned());
        assert_eq!(version_1, version_2)
    }

    #[test]
    fn should_get_unset_version() {
        let cli = clients::Cli::default();
        let node = cli.run(images::redis::Redis::default());
        let mut storage = RedisRefreshTokenStorage::new(
            "127.0.0.1".to_owned(),
            node.get_host_port(6379).unwrap(),
            1,
        )
        .unwrap();

        let version = storage.get_version(&"user-a".to_owned(), &"device-a".to_owned());
        assert_eq!(version, 0)
    }

    #[test]
    fn should_clear_after_timeout() {
        let cli = clients::Cli::default();
        let node = cli.run(images::redis::Redis::default());
        let mut storage = RedisRefreshTokenStorage::new(
            "127.0.0.1".to_owned(),
            node.get_host_port(6379).unwrap(),
            1,
        )
        .unwrap();

        storage.new_version(&"user-a".to_owned(), &"device-a".to_owned());
        thread::sleep(Duration::from_secs(2));
        let version = storage.get_version(&"user-a".to_owned(), &"device-a".to_owned());
        assert_eq!(version, 0)
    }

    #[test]
    fn should_clear_if_so_much_user_id() {
        let cli = clients::Cli::default();
        let node = cli.run(images::redis::Redis::default());
        let mut storage = RedisRefreshTokenStorage::new(
            "127.0.0.1".to_owned(),
            node.get_host_port(6379).unwrap(),
            1,
        )
        .unwrap();

        storage.new_version(&"user-a".to_owned(), &"device-a".to_owned());
        for i in 0..RedisRefreshTokenStorage::COUNT_DEVICES_PER_USER + 1 {
            storage.new_version(&"user-a".to_owned(), &format!("device-{}", i));
        }

        let version = storage.get_version(&"user-a".to_owned(), &"device-a".to_owned());
        assert_eq!(version, 0)
    }

    #[test]
    fn should_truncate_device_id() {
        let cli = clients::Cli::default();
        let node = cli.run(images::redis::Redis::default());
        let mut storage = RedisRefreshTokenStorage::new(
            "127.0.0.1".to_owned(),
            node.get_host_port(6379).unwrap(),
            1,
        )
        .unwrap();

        storage.new_version(
            &"user-a".to_owned(),
            &"012345678901234567890123456789".to_owned(),
        );
        let version = storage.get_version(&"user-a".to_owned(), &"01234567890123456789".to_owned());
        assert_eq!(version, 1)
    }
}
