-- привязка пользователя к cookie
create table cookie_users (
	user_id bigint not null primary key,
	cookie char(128) not null,
	linked bool default false
);

create unique index cookie_users_cookie_uindex on cookie_users (cookie);

alter table cookie_users
	add constraint cookie_users_id_fk foreign key (user_id) references users (id);

