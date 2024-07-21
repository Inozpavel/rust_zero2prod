-- Add migration script here
CREATE TABLE IF NOT EXISTS subscription_tokens
(
    subscription_token text NOT NULL PRIMARY KEY,
    subscriber_id text NOT NULL
)