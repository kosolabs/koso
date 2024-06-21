-- Add migration script here
-- Add a table update notification function
CREATE OR REPLACE FUNCTION table_update_notify() RETURNS trigger AS $$
DECLARE
  row_record RECORD;
  old_record RECORD;
BEGIN
  CASE TG_OP
  WHEN 'UPDATE' THEN
     row_record := NEW;
     old_record := OLD;
  WHEN 'INSERT' THEN
     row_record := NEW;
  WHEN 'DELETE' THEN
     row_record := OLD;
  ELSE
     RAISE EXCEPTION 'Unknown TG_OP: "%". Should not occur!', TG_OP;
  END CASE;

  PERFORM pg_notify(
    'table_update',
    json_build_object(
      'timestamp', CURRENT_TIMESTAMP,
      'action', UPPER(TG_OP),
      'table', TG_TABLE_NAME,
      'id', row_record.id,
      -- Include the entire new and old rows.
      -- This doesn't scale though, there's a 3KB size limit on notify
      'record', row_to_json(row_record)::text,
      'old', row_to_json(old_record)::text
    )::text
  );

  RETURN row_record;
END;
$$ LANGUAGE plpgsql;

-- Add UPDATE row trigger
CREATE OR REPLACE TRIGGER tasks_notify_update AFTER UPDATE ON tasks FOR EACH ROW EXECUTE PROCEDURE table_update_notify();

-- Add INSERT row trigger
CREATE OR REPLACE TRIGGER tasks_notify_insert AFTER INSERT ON tasks FOR EACH ROW EXECUTE PROCEDURE table_update_notify();

-- Add DELETE row trigger
CREATE OR REPLACE TRIGGER tasks_notify_delete AFTER DELETE ON tasks FOR EACH ROW EXECUTE PROCEDURE table_update_notify();