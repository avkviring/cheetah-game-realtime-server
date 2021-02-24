------------------------------------
-- привязка пользователя к google
------------------------------------
create table google_players
(
	player bigint not null primary key,
	ip     inet not null,
	email  varchar(255) not null
);

create unique index google_players_email_uindex on google_players (email);

alter table google_players
	add constraint google_players_id_fk foreign key (player) references players (id);

------------------------------------
-- история привязки пользователей
-- для разбора ситуаций угона
------------------------------------
create table google_players_history
(
	id     bigserial primary key,
	ip     inet not null,
	time   timestamp default CURRENT_TIMESTAMP not null,
	player bigint not null,
	email  varchar(255) not null
);

create index google_players_history_email_index on google_players_history (email);
create index google_players_history_players_index on google_players_history (player);
