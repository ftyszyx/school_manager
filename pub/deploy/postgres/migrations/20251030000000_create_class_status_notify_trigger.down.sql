-- Drop the trigger
DROP TRIGGER IF EXISTS class_status_change_trigger ON classes;

-- Drop the function
DROP FUNCTION IF EXISTS notify_class_status_change();
