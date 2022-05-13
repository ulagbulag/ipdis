-- Your SQL goes here
CREATE TABLE dynpaths (
  id SERIAL PRIMARY KEY,
  name VARCHAR NOT NULL,
  path VARCHAR(44) NOT NULL,  -- SHA256 + base58
  len BIGINT NOT NULL
)
