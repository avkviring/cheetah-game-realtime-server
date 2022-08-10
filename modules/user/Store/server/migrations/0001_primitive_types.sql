create table cheetah_user_store_long_value
(
	user_uuid  uuid not null,
	field_name varchar(128) not null,
	value      bigint
);
create unique index cheetah_user_store_long_value_index on cheetah_user_store_long_value (user_uuid, field_name);


create table cheetah_user_store_double_value
(
	user_uuid  uuid not null,
	field_name varchar(128) not null,
	value      double precision
);
create unique index cheetah_user_store_double_value_index on cheetah_user_store_double_value (user_uuid, field_name);


create table cheetah_user_store_string_value
(
	user_uuid  uuid not null,
	field_name varchar(128) not null,
	value      text,
	primary key (user_uuid, field_name)
);
create unique index cheetah_user_store_string_value_index on cheetah_user_store_string_value (user_uuid, field_name);
