------------------------------------
-- привязка пользователя к cookie
------------------------------------
create table cookie_players
(
	player bigint not null primary key,
	cookie char(128) not null
);

create unique index cookie_players_cookie_uindex on cookie_players (cookie);

alter table cookie_players
	add constraint cookie_players_id_fk foreign key (player) references players (id);

