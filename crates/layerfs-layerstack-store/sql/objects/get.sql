-- family: objects
-- name: get
-- parameters: ?1 ObjectId
-- results: bytes
SELECT canonical_length, pack_id, group_number, record_number FROM objects WHERE object_id = ?1;
