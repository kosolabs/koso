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
