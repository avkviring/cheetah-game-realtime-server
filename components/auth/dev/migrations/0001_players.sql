---------------
-- игроки
---------------
create table players
(
	id   bigserial not null constraint players_pk primary key,
	ip   inet not null,
	time timestamp default CURRENT_TIMESTAMP not null
);


create
unique index players_id_uindex on players (id);

