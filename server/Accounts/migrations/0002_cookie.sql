create table cookie_users
(
	user_id bigint REFERENCES users (id),
	cookie  char(128) not null,
	linked  bool default false
);

create unique index cookie_users_cookie_index on cookie_users (cookie);
