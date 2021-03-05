------------------------------------
-- привязка пользователя к google
------------------------------------
create table google_players
(
	player bigint not null primary key,
	ip     inet not null,
	google_id  varchar(255) not null
);

create unique index google_players_googleid_uindex on google_players (google_id);

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
	google_id  varchar(255) not null
);

create index google_players_history_googleid_index on google_players_history (google_id);
create index google_players_history_players_index on google_players_history (player);
