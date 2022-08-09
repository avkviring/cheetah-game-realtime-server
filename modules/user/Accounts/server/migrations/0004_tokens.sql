create table cheetah_user_accounts_tokens
(
	user_uuid uuid,
	device    varchar(255),
	token     uuid,
	create_at timestamp default CURRENT_TIMESTAMP not null
);
create unique index cheetah_user_accounts_tokens_index on cheetah_user_accounts_tokens (user_uuid, device, token);
