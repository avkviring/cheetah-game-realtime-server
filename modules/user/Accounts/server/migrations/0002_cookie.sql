create table cheetah_user_accounts_cookie
(
	user_uuid uuid primary key not null,
	cookie    uuid
);

create unique index cheetah_user_accounts_cookie_index on cheetah_user_accounts_cookie (cookie);
