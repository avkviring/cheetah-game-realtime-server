create table user_long_value (
    user_uuid String,
    field_name Utf8,
    value Int64,
    primary key (user_uuid, field_name)
);

create table user_double_value (
    user_uuid String,
    field_name Utf8,
    value Double,
    primary key (user_uuid, field_name)
)
