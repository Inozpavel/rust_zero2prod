-- Add migration script here
CREATE TABLE users
(
    user_id uuid NOT NULL PRIMARY KEY,
    username text NOT NULL,
    password text NOT NULL
)