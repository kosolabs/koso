UPDATE plugin_configs
SET settings = settings - 'type'
WHERE settings ? 'type';
