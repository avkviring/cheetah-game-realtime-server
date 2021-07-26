-- игроки
create table users (
	id          bigserial not null constraint users_pk primary key,
	ip          inet not null,
	create_time timestamp default CURRENT_TIMESTAMP not null
);

create index users_id_uindex on users (id);
