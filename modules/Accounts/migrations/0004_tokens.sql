-- привязка пользователя к google
create table tokens
(
	user   String,
	device Utf8,
	token_uuid  String,
	create_at Timestamp,
	PRIMARY KEY (user, device)
)