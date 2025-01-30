CREATE TABLE user_notification_configs (
    email varchar(320) NOT NULL,
    notifier varchar(64) NOT NULL,
    enabled boolean NOT NULL,
    settings jsonb NOT NULL,
    PRIMARY KEY (email, notifier)
);
