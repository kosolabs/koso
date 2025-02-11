UPDATE user_notification_configs
SET settings = '{"type": "telegram"}'::jsonb || settings;
