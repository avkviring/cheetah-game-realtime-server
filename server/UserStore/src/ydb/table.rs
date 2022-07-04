pub static COLUMN_USER: &str = "user_uuid";
pub static COLUMN_FIELD_NAME: &str = "field_name";
pub static COLUMN_FIELD_VALUE: &str = "value";

pub static LONG_TABLE: &str = "user_long_value";
pub static DOUBLE_TABLE: &str = "user_double_value";
pub static STRING_TABLE: &str = "user_string_value";

pub trait ToDbTable {
	fn to_db_table() -> &'static str;
}

impl ToDbTable for i64 {
	fn to_db_table() -> &'static str {
		LONG_TABLE
	}
}

impl ToDbTable for f64 {
	fn to_db_table() -> &'static str {
		DOUBLE_TABLE
	}
}

impl ToDbTable for &str {
	fn to_db_table() -> &'static str {
		STRING_TABLE
	}
}

impl ToDbTable for String {
	fn to_db_table() -> &'static str {
		STRING_TABLE
	}
}
