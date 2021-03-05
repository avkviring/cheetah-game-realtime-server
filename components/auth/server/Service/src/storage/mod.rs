pub mod cookie;
pub mod google;
pub mod pg;
pub mod players;

#[cfg(test)]
pub mod test {
    use std::collections::HashMap;

    use testcontainers::clients::Cli;
    use testcontainers::images::postgres::Postgres;
    use testcontainers::{images, Container, Docker};

    use crate::storage::pg::PgStorage;

    pub async fn setup_postgresql_storage<'a>(
        cli: &'a Cli,
    ) -> (PgStorage, Container<'a, Cli, Postgres>) {
        let mut env = HashMap::default();
        env.insert("POSTGRES_USER".to_owned(), "auth".to_owned());
        env.insert("POSTGRES_PASSWORD".to_owned(), "passwd".to_owned());
        let image = images::postgres::Postgres::default()
            .with_version(12)
            .with_env_vars(env);
        let node = cli.run(image);
        let port = node.get_host_port(5432).unwrap();
        let storage = PgStorage::new("auth", "passwd", "127.0.0.1", port).await;
        (storage, node)
    }
}
