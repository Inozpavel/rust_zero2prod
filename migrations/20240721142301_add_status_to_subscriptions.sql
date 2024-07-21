-- Add migration script here
ALTER TABLE subscriptions ADD COLUMN status text NULL;

BEGIN;
UPDATE subscriptions
SET status = 'confirmed'
WHERE status IS NULL;
ALTER TABLE subscriptions ALTER COLUMN status SET NOT NULL;
COMMIT;

ALTER TABLE subscriptions ALTER COLUMN status SET NOT NULL;