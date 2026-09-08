-- family: objects
-- name: insert
-- parameters: ?1 ObjectId, ?2 canonical length, ?3 pack ID, ?4 group, ?5 record
-- affected rows: one inserted; conflicts fail
INSERT INTO objects(object_id, canonical_length, pack_id, group_number, record_number)
VALUES (?1, ?2, ?3, ?4, ?5);
