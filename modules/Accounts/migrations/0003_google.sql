-- привязка пользователя к google
create table google_users
(
	user      String,
	google_id Utf8,
	INDEX user GLOBAL ON (user),
	PRIMARY KEY (google_id)
);

-- история привязки пользователей
-- для разбора ситуаций угона
create table google_users_history
(
	user      String,
	google_id Utf8,
	time      timestamp,
	PRIMARY KEY (user, google_id, time)
);
