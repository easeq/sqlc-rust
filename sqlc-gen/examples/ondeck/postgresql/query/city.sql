-- name: ListCities :many
select *
from city
order by name
;

-- name: GetCity :one
select *
from city
where slug = $1
;

-- name: CreateCity :one
-- Create a new city. The slug must be unique.
-- This is the second line of the comment
-- This is the third line
INSERT INTO city (
    name,
    slug
) VALUES (
    $1,
    $2
) RETURNING *;

-- name: UpdateCityName :exec
UPDATE city
SET name = $2
WHERE slug = $1;
