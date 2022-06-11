pub use std::collections::HashMap;

use include_dir::Dir;
use sha2::{Digest, Sha256};
use ydb::Client;
use ydb::TableClient;

use crate::{query, select, update};

///
/// Поддержка миграций для ydb
/// Пока достаточно примитивно - только прямой проход, без откатов, однако возможно они и не нужны
///
#[derive(Clone, Default)]
#[allow(unused)]
pub struct Migrator {
	migrations: Vec<Migration>,
}

#[derive(Debug)]
pub enum MigrationError {
	FailCreateMigrationTable(String),
	FailGetAppliedMigrations(String),
	FailApplyMigration(String),
	ChangeAppliedMigration(String),
	FailMarkMigrationAsApplied(String),
}

impl Migrator {
	#[allow(unused)]
	pub fn new_from_dir(dir: &Dir) -> Self {
		let mut migration = Self::default();
		dir.files().for_each(|f| {
			let file_name = f.path().file_name().unwrap().to_str().unwrap().to_string();
			if file_name.ends_with(".sql") {
				let content = f.contents_utf8().unwrap().to_owned();
				migration.add_migration(file_name, content);
			}
		});
		migration
	}

	#[allow(unused)]
	pub fn add_migration(&mut self, name: String, sql: String) {
		let migration = Migration::new(name, sql);
		self.migrations.push(migration);
	}

	#[allow(unused)]
	pub async fn migrate(&mut self, client: &mut Client) -> Result<(), MigrationError> {
		let table_client = client.table_client();
		Self::create_migrated_table(&table_client).await;
		let applied_migrations = Self::get_applied_migrations(&table_client).await?;
		self.migrations.sort_by_key(|m| m.name.clone());
		self.verify_migrations(&applied_migrations)?;
		self.apply_migrate(table_client, applied_migrations).await
	}

	async fn get_applied_migrations(
		table_client: &TableClient,
	) -> Result<HashMap<String, AppliedMigration>, MigrationError> {
		let result: Vec<(String, Vec<u8>)> =
			select!(table_client, query!("select * from migrations"),
			name=> String,
			checksum=> ydb::Bytes)
			.await
			.map_err(|e| MigrationError::FailGetAppliedMigrations(format!("{}", e)))?;

		Ok(result
			.into_iter()
			.map(|(name, checksum)| (name.clone(), AppliedMigration { name, checksum }))
			.collect())
	}
	fn verify_migrations(
		&self,
		applied_migrations: &HashMap<String, AppliedMigration>,
	) -> Result<(), MigrationError> {
		for migration in self.migrations.iter() {
			if let Some(applied_migration) = applied_migrations.get(&migration.name) {
				if applied_migration.checksum != migration.checksum {
					return Err(MigrationError::ChangeAppliedMigration(format!(
						"name {}",
						migration.name
					)));
				}
			}
		}
		Ok(())
	}

	async fn apply_migrate(
		&mut self,
		table_client: TableClient,
		applied_migrations: HashMap<String, AppliedMigration>,
	) -> Result<(), MigrationError> {
		for migration in &self.migrations {
			if applied_migrations.contains_key(&migration.name) {
				continue;
			}
			Self::apply_migration(&table_client, migration).await?;
			Self::mark_migration_as_applied(&table_client, migration).await?;
		}
		Ok(())
	}

	async fn mark_migration_as_applied(
		table_client: &TableClient,
		migration: &Migration,
	) -> Result<(), MigrationError> {
		update!(
			table_client,
			query!(
				"insert into migrations (name, checksum) values($name, $checksum)",
				name => migration.name,
				checksum => migration.checksum
			)
		)
		.await
		.map_err(|e| MigrationError::FailMarkMigrationAsApplied(format!("{}", e)))
	}

	async fn apply_migration(
		table_client: &TableClient,
		migration: &Migration,
	) -> Result<(), MigrationError> {
		table_client
			.retry_execute_scheme_query(migration.sql.clone())
			.await
			.map_err(|e| MigrationError::FailApplyMigration(format!("{}", e)))
	}

	async fn create_migrated_table(table_client: &TableClient) -> Result<(), MigrationError> {
		table_client
			.retry_execute_scheme_query(
				"create table migrations(name Utf8, checksum string,PRIMARY KEY(name));",
			)
			.await
			.map_err(|e| MigrationError::FailCreateMigrationTable(format!("{}", e)))
	}
}

#[allow(unused)]
struct AppliedMigration {
	name: String,
	checksum: Vec<u8>,
}

#[derive(Clone)]
struct Migration {
	name: String,
	sql: String,
	checksum: Vec<u8>,
}

impl Migration {
	fn new(name: String, sql: String) -> Self {
		let mut hasher = Sha256::default();
		hasher.update(sql.clone());
		let checksum = hasher.finalize().to_vec();
		Self {
			name,
			sql,
			checksum,
		}
	}
}

#[cfg(test)]
mod tests {
	use include_dir::include_dir;
	use ydb::Query;

	use crate::migration::Migrator;
	use crate::test_container::get_or_create_ydb_instance;

	#[tokio::test]
	async fn should_migration() {
		let (_node, mut client) = get_or_create_ydb_instance("should_migration").await;
		let mut migrator = Migrator::default();

		migrator.add_migration(
			"002.sql".to_owned(),
			"alter table a ADD COLUMN some_flag Bool;".to_owned(),
		);
		migrator.add_migration(
			"001.sql".to_owned(),
			"create table a(id int, PRIMARY KEY(id));".to_owned(),
		);
		migrator.migrate(&mut client).await.unwrap();
		client
			.table_client()
			.retry_transaction(|mut t| async move {
				t.query(Query::new(
					"insert into a (id, some_flag) values (1, false)",
				))
				.await?;
				Ok(())
			})
			.await
			.unwrap();
	}

	#[tokio::test]
	async fn should_not_migration_if_migrated() {
		let (_node, mut client) =
			get_or_create_ydb_instance("should_not_migration_if_migrated").await;
		let mut migrator = Migrator::default();
		migrator.add_migration(
			"001.sql".to_owned(),
			"create table a(id int, PRIMARY KEY(id));".to_owned(),
		);
		migrator.migrate(&mut client).await.unwrap();
		migrator.migrate(&mut client).await.unwrap();
	}

	#[tokio::test]
	async fn should_not_migration_when_already_migrated_script_changed() {
		let (_node, mut client) =
			get_or_create_ydb_instance("should_not_migration_when_already_migrated_script_changed")
				.await;
		{
			let mut migrator = Migrator::default();
			migrator.add_migration(
				"001.sql".to_owned(),
				"create table a(id int, PRIMARY KEY(id));".to_owned(),
			);
			migrator.migrate(&mut client).await.unwrap();
		}

		let mut migrator = Migrator::default();
		migrator.add_migration(
			"001.sql".to_owned(),
			"create table b(id int, PRIMARY KEY(id));".to_owned(),
		);
		assert!(migrator.migrate(&mut client).await.is_err());
	}

	#[tokio::test]
	async fn should_migration_from_directory() {
		let (_node, mut client) =
			get_or_create_ydb_instance("should_migration_from_directory").await;
		let mut migrator =
			Migrator::new_from_dir(&include_dir!("$CARGO_MANIFEST_DIR/test-migration"));
		migrator.migrate(&mut client).await.unwrap();
		client
			.table_client()
			.retry_transaction(|mut t| async move {
				t.query(Query::new("insert into a (id) values (1)")).await?;
				t.query(Query::new("insert into b (id) values (1)")).await?;
				Ok(())
			})
			.await
			.unwrap();
	}
}
