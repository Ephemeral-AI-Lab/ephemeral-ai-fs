-- family: query
-- name: canonical_storage
-- parameters: none
-- results: selected object count and canonical bytes
SELECT count(*), COALESCE(sum(canonical_length), 0) FROM objects;
