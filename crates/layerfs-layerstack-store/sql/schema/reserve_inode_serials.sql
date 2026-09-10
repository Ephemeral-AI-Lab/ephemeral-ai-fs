-- family: schema
-- name: reserve_inode_serials
-- parameters: scope, count
INSERT INTO scope_allocator(scope, highwater) VALUES (?1, ?2)
ON CONFLICT(scope) DO UPDATE SET highwater = highwater + ?2
WHERE highwater <= 9223372036854775807 - ?2
RETURNING highwater;
