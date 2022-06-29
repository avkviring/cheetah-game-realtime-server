create table user_long_list (
    user_uuid String,
    set_name Utf8,
    value Int64,
    primary key (user_uuid, set_name, value)
);