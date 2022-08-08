-- привязка пользователя к google
create table cheetah_user_accounts_google
(
	user_uuid uuid primary key not null,
	google_id varchar(255) not null
);

create unique index cheetah_user_accounts_google_googl_id_index on cheetah_user_accounts_google (google_id);

-- история привязки пользователей
-- для разбора ситуаций угона
create table cheetah_user_accounts_google_users_history
(
	user_uuid uuid not null,
	google_id varchar(255) not null,
	time      timestamp default CURRENT_TIMESTAMP not null
);

create unique index cheetah_user_accounts_google_users_history_google_id_index on cheetah_user_accounts_google (google_id);
