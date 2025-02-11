UPDATE plugin_configs
SET settings = '{"type": "github"}'::jsonb || settings;
