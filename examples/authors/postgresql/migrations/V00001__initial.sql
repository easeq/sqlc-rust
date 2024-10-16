CREATE TABLE authors (
  id   BIGSERIAL PRIMARY KEY,
  uuid uuid DEFAULT gen_random_uuid(),
  name text      NOT NULL,
  bio  text,
  data json,
  ip_inet inet NOT NULL DEFAULT '0.0.0.0'::inet,
  ip_cidr inet NOT NULL DEFAULT '0.0.0.0/24'::cidr,
  mac_address macaddr NOT NULL DEFAULT '00-00-00-00-00-000'::macaddr,
  created_at timestamptz NOT NULL DEFAULT (now()),
  updated_at timestamptz NOT NULL DEFAULT (now())
);
