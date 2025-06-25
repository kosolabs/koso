DROP TABLE subscriptions;

ALTER TABLE users
ADD COLUMN premium BOOLEAN NOT NULL DEFAULT FALSE;

UPDATE users
SET premium=subscription_end_time>now();

ALTER TABLE users
DROP COLUMN subscription_end_time;
