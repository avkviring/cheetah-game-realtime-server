-- игроки
create table users
(
	id          bigserial PRIMARY KEY,
	ip          inet not null,
	create_time timestamp default CURRENT_TIMESTAMP not null
);
