-- name: GetAuthor :one
select *
from authors
where author_id = $1
;

-- name: DeleteBookExecResult :execresult
delete from books
where book_id = $1
;

-- name: DeleteBook :batchexec
delete from books
where book_id = $1
;

-- name: DeleteBookNamedFunc :batchexec
delete from books
where book_id = sqlc.arg(book_id)
;

-- name: DeleteBookNamedSign :batchexec
delete from books
where book_id = @book_id
;

-- name: BooksByYear :batchmany
select *
from books
where year = $1
;

-- name: CreateAuthor :one
INSERT INTO authors (name) VALUES ($1)
RETURNING *;

-- name: CreateBook :batchone
INSERT INTO books (
  author_id,
  isbn,
  book_type,
  title,
  year,
  available,
  tags
) VALUES (
  $1,
  $2,
  $3,
  $4,
  $5,
  $6,
  $7
)
RETURNING *;

-- name: UpdateBook :batchexec
UPDATE books
SET title = $1, tags = $2
WHERE book_id = $3;

-- name: GetBiography :batchone
select biography
from authors
where author_id = $1
;
