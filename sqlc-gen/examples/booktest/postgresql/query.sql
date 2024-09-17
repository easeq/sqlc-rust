-- name: GetAuthor :one
select *
from authors
where author_id = $1
;

-- name: GetBook :one
select *
from books
where book_id = $1
;

-- name: DeleteBook :exec
delete from books
where book_id = $1
;

-- name: BooksByTitleYear :many
select *
from books
where title = $1 and year = $2
;

-- name: BooksByTags :many
select book_id, title, name, isbn, tags
from books
left join authors on books.author_id = authors.author_id
where tags && $1::varchar[]
;

-- name: CreateAuthor :one
INSERT INTO authors (name) VALUES ($1)
RETURNING *;

-- name: CreateBook :one
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

-- name: UpdateBook :exec
UPDATE books
SET title = $1, tags = $2
WHERE book_id = $3;

-- name: UpdateBookISBN :exec
UPDATE books
SET title = $1, tags = $2, isbn = $4
WHERE book_id = $3;

-- name: SayHello :one
select *
from say_hello($1)
;
