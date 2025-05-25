CREATE RANGE INDEX audiobook_last_upload_idx IF NOT EXISTS
FOR (a:Audiobook)
ON (a.last_upload);

CREATE CONSTRAINT constraint_path_is_unique IF NOT EXISTS
FOR (n:Audiobook)
REQUIRE n.path IS UNIQUE;
