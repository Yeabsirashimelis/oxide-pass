-- Mark all apps that are not STOPPED as STOPPED to clean up stale records
UPDATE apps SET status = 'STOPPED'::app_status, pid = NULL WHERE status != 'STOPPED'::app_status;
