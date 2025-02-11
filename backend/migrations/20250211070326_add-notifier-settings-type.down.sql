UPDATE user_notification_configs
SET settings = settings - 'type'
WHERE settings ? 'type';
