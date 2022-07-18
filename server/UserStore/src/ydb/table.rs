pub static COLUMN_USER: &str = "user_uuid";
pub static COLUMN_FIELD_NAME: &str = "field_name";
pub static COLUMN_FIELD_VALUE: &str = "value";

pub static LONG_TABLE: &str = "user_long_value";
pub static DOUBLE_TABLE: &str = "user_double_value";
pub static STRING_TABLE: &str = "user_string_value";

pub fn ydb_type_to_table_name(typ: &str) -> &'static str {
	match typ {
		"Int64" => LONG_TABLE,
		"Double" => DOUBLE_TABLE,
		"String" => STRING_TABLE,
		t => panic!("Value of YDB type {} cannot be stored in UserStore", t),
	}
}
