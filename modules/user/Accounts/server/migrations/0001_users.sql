create table cheetah_user_account_users
(
	user_uuid   uuid primary key not null,
	create_date timestamp default CURRENT_TIMESTAMP not null
)
