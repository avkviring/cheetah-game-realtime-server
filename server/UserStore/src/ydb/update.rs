use ydb::TableClient;

pub struct YDBUpdate {
	client: TableClient,
}

impl YDBUpdate {
	pub fn new(client: TableClient) -> Self {
		Self { client }
	}
}
