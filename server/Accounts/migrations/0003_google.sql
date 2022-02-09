-- привязка пользователя к google
create table google_users
(
	user_id   bigint REFERENCES users (id),
	ip        inet not null,
	google_id varchar(255) not null
);

create unique index google_users_google_id_uindex on google_users (google_id);

-- история привязки пользователей
-- для разбора ситуаций угона
create table google_users_history
(
	id        bigserial primary key,
	ip        inet not null,
	time      timestamp default CURRENT_TIMESTAMP not null,
	user_id   bigint not null,
	google_id varchar(255) not null
);

create index google_users_history_google_id_index on google_users_history (google_id);
create index google_users_history_users_index on google_users_history (user_id);
