create table cheetah_user_accounts_tokens
(
	user_uuid uuid primary key not null,
	device    varchar(255),
	token     uuid,
	create_at timestamp default CURRENT_TIMESTAMP not null
)