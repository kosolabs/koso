ALTER TABLE users
ADD COLUMN stripe_customer_id text,
ADD COLUMN premium_subscription_end timestamptz,
ADD COLUMN premium_subscription_seats integer;
