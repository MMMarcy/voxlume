CREATE RANGE INDEX audiobook_last_upload_idx
FOR (a:Audiobook)
ON (a.last_upload);
