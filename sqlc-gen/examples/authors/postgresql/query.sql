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

-- name: DeleteAuthor :exec
delete from authors
where id = $1
;
