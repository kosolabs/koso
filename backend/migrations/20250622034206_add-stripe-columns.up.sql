CREATE TABLE subscriptions (
    email varchar(320) NOT NULL,
    stripe_customer_id text NOT NULL,
    end_time timestamptz NOT NULL,
    seats integer NOT NULL,
    member_emails varchar(320)[] NOT NULL,
    PRIMARY KEY (email)
);

ALTER TABLE users
ADD COLUMN subscription_end_time timestamptz;

UPDATE users
SET subscription_end_time=TIMESTAMP '2100-01-20 13:00:00';

ALTER TABLE users
DROP COLUMN premium;
