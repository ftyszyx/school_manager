-- Function to send a notification when class status changes
CREATE OR REPLACE FUNCTION notify_class_status_change()
RETURNS TRIGGER AS $$
BEGIN
  IF OLD.status IS DISTINCT FROM NEW.status THEN
    PERFORM pg_notify(
      'class_status_updates',
      json_build_object(
        'school_id', NEW.school_id,
        'grade', NEW.grade,
        'class', NEW.class,
        'class_id', NEW.id,
        'new_status', NEW.status
      )::text
    );
  END IF;
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger to call the function on update
CREATE TRIGGER class_status_change_trigger
AFTER UPDATE OF status ON classes
FOR EACH ROW
EXECUTE FUNCTION notify_class_status_change();
