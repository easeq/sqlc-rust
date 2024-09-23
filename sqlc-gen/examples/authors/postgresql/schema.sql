CREATE TABLE authors (
          id   BIGSERIAL PRIMARY KEY,
          name text      NOT NULL,
          bio  text
);

CREATE TABLE IF NOT EXISTS "site" (
  id uuid PRIMARY KEY,
  "url" varchar NOT NULL UNIQUE,
  "status" boolean NOT NULL DEFAULT true,
  "data" json NOT NULL,
  "inet" inet NOT NULL,
  "mac" macaddr NOT NULL,
  "new_id" uuid NOT NULL,
  "created_at" timestamptz NOT NULL DEFAULT (now()),
  "updated_at" timestamptz NOT NULL DEFAULT (now())
);
