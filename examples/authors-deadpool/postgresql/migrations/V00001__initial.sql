DROP TYPE IF EXISTS type_genre;
CREATE TYPE type_genre as ENUM (
  'history',
  'Children',
  'cLaSSic',
  'ADVENTURE'
);

CREATE TABLE authors (
  id   BIGSERIAL PRIMARY KEY,
  uuid uuid DEFAULT gen_random_uuid(),
  name text      NOT NULL,
  genre type_genre NOT NULL DEFAULT 'ADVENTURE',
  bio  text,
  data json,
  attrs hstore,
  ip_inet inet NOT NULL DEFAULT '0.0.0.0'::inet,
  ip_cidr cidr NOT NULL DEFAULT '0.0.0.0/24'::cidr,
  mac_address macaddr NOT NULL DEFAULT '00-00-00-00-00-000'::macaddr,
  geo_point point,
  geo_rect box, 
  geo_path path, 
  bit_a bit(3),
  varbit_a bit varying(5),
  created_at timestamptz NOT NULL DEFAULT (now()),
  updated_at timestamptz NOT NULL DEFAULT (now())
);
