-- name: GetAuthor :one
select *
from authors
where id = $1
limit 1
;

-- name: ListAuthors :many
select *
from authors
order by name
;

-- name: CreateAuthor :one
INSERT INTO authors (
  name, bio
) VALUES (
  $1, $2
)
RETURNING *;

-- name: CreateAuthorFull :one
INSERT INTO authors (
  name, 
  bio,
  data,
  genre,
  attrs,
  ip_inet,
  ip_cidr,
  mac_address,
  geo_point,
  geo_rect,
  geo_path,
  bit_a,
  varbit_a,
  created_at,
  updated_at
) VALUES (
  $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15
)
RETURNING *;

-- name: DeleteAuthor :exec
delete from authors
where id = $1
;
