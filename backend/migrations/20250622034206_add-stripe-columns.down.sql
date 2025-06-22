ALTER TABLE users
DROP COLUMN stripe_customer_id,
DROP COLUMN premium_subscription_end,
DROP COLUMN premium_subscription_seats;
