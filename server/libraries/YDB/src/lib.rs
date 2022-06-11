use include_dir::Dir;
use ydb::{Client, ClientBuilder, StaticDiscovery, YdbError, YdbOrCustomerError};
use ydb_grpc::ydb_proto::status_ids::StatusCode;

use crate::migration::Migrator;

pub mod converters;
pub mod migration;
#[cfg(feature = "test_container")]
pub mod test_container;


#[macro_export]
macro_rules! update {
	($ydb:expr, $query:expr) => {{
		$ydb.retry_transaction(|mut t| async move {
			t.query($query).await?;
			t.commit().await?;
			Ok(())
		})
	}};
}

#[macro_export]
macro_rules! query {
	($sql:expr) => {{
			ydb::Query::new($sql)
	}};
	($sql:expr, $($name:ident => $value:expr),*) => {{
			let mut query = String::new();
			let mut params: std::collections::HashMap<String, ydb::Value> = Default::default();
		  	$(
				{
					let value = &$value;
					let name = stringify!($name);
					let type_name = $crate::converters::YDBValueConverter::get_type_name(value);
					let ydb_value =$crate::converters::YDBValueConverter::to_ydb_value(value);
		  			params.insert(format!("${}",name), ydb_value);
		  			query.push_str(format!("declare ${} as {};\n",name,type_name).as_str());
				}
		  	)*
			ydb::Query::new(format!("{}\n{}",query, $sql)).with_params(params)
	}};
}

#[macro_export]
macro_rules! select {
	($ydb:expr, $query:expr, $($name:ident => $type:ty),* ) => {{
		$ydb.retry_transaction(|mut t| async move {
			let query_result = t.query($query).await?;
			let result_set = query_result.into_only_result()?;
			let result:Vec<_> = result_set.rows().map(|mut row| {
				$(
					let $name:Option<$type>  = row.remove_field_by_name(stringify!($name)).unwrap().try_into().unwrap();
				)*
				($($name.unwrap().into()),*)
			}).collect();
			Ok(result)
		})
	}};
}

pub fn is_unique_violation_error(result: &YdbOrCustomerError) -> bool {
	matches!(result, 
		YdbOrCustomerError::YDB(YdbError::YdbStatusError(status)) 
		if status.operation_status().unwrap() == StatusCode::PreconditionFailed)
}

pub async fn connect_to_ydb_and_prepare_schema<'a>(namespace:&str,
												   db: &str,
												   host: &str,
												   port: u16,
												   migration_sql_dir: &Dir<'a>
) -> Client {
	let connection_url = format!("grpc://{}:{}", host, port);
	let db = format!("{}/{}", namespace, db);
	create_db(connection_url.as_str(), namespace, db.as_str()).await;
	let mut client = create_client(connection_url.as_str(), db.as_str()).await;
	Migrator::new_from_dir(migration_sql_dir)
		.migrate(&mut client)
		.await
		.unwrap();
	client
}

async fn create_client(connection_url: &str, db: &str) -> Client {
	let discovery = StaticDiscovery::from_str(connection_url).unwrap();
	let client = ClientBuilder::from_str(connection_url)
		.unwrap()
		.with_database(db)
		.with_discovery(discovery)
		.client()
		.unwrap();
	client.wait().await.unwrap();
	client
}


async fn create_db(connection_url: &str, root_namespace:&str, db: &str) {
	{
		let client = create_client(connection_url, root_namespace).await;
		client.wait().await.unwrap();
		client
			.scheme_client()
			.make_directory(db.to_owned())
			.await
			.unwrap();
	}
}


#[cfg(test)]
pub mod tests {
	use include_dir::include_dir;

	use crate::migration::Migrator;
	use crate::test_container::get_or_create_ydb_instance;

	#[tokio::test]
	async fn should_create_query() {
		let (_node, mut client) = get_or_create_ydb_instance("should_create_query").await;
		let mut migrator = Migrator::new_from_dir(&include_dir!("$CARGO_MANIFEST_DIR/test-migration"));
		migrator.migrate(&mut client).await.unwrap();

		let id = 124;
		update!(client.table_client(), query!("insert into a (id) values($id)", id=>id))
			.await
			.unwrap();

		let result: Vec<i32> = select!(client.table_client(), query!("select * from a"), id=>i32)
			.await
			.unwrap();

		assert!(result.iter().any(|v| *v == id));
	}
}
