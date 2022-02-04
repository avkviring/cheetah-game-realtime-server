-- игроки
create table users (
	id          bigserial not null constraint users_pk primary key,
	ip          inet not null,
	create_time timestamp default CURRENT_TIMESTAMP not null
);

create index users_id_uindex on users (id);
-- привязка пользователя к google
create table google_users (
							  user_id 	bigint not null primary key,
							  ip     		inet not null,
							  google_id  	varchar(255) not null
);

create unique index google_users_googleid_uindex on google_users (google_id);

-- история привязки пользователей
-- для разбора ситуаций угона
create table google_users_history (
									  id     		bigserial primary key,
									  ip     		inet not null,
									  time   		timestamp default CURRENT_TIMESTAMP not null,
									  user_id 	bigint not null,
									  google_id  	varchar(255) not null
);

create index google_users_history_googleid_index on google_users_history (google_id);
create index google_users_history_users_index on google_users_history (user_id);
-- привязка пользователя к cookie
create table cookie_users (
							  user_id bigint not null primary key,
							  cookie char(128) not null,
							  linked bool default false
);

create unique index cookie_users_cookie_uindex on cookie_users (cookie);
