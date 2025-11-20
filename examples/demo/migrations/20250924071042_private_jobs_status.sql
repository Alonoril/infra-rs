-- Add migration script here

DROP TABLE IF EXISTS test_users;
CREATE TABLE IF NOT EXISTS test_users
(
    id          INTEGER   NOT NULL PRIMARY KEY AUTOINCREMENT,
	password      VARCHAR      NOT NULL UNIQUE,
	access_token TEXT,
    created_at TIMESTAMP not null default current_timestamp,
    updated_at TIMESTAMP not null default current_timestamp
);