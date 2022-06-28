mod fetch;
mod update;

pub use update::YDBUpdate;

static USER: &str = "user_uuid";
static FIELD_NAME: &str = "field_name";
static FIELD_VALUE: &str = "value";

static LONG_TABLE: &str = "user_long_value";
