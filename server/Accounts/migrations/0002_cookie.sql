create table cookie_users
(
	cookie String,
	user   String,		
	INDEX  user GLOBAL ON (user),
	PRIMARY KEY (cookie)
);
