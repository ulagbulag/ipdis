-- Your SQL goes here
CREATE TABLE idf_words (
  id SERIAL PRIMARY KEY,
  kind SHA256HASH NOT NULL,
  word SHA256HASH NOT NULL,
  count BIGINT NOT NULL,
  UNIQUE (kind, word)
);

CREATE TABLE idf_logs (
  id SERIAL PRIMARY KEY,
  account ACCOUNT NOT NULL,  -- METADATA, 
  signature SIGNATURE NOT NULL UNIQUE,  -- METADATA, 
  created_date TIMESTAMP NOT NULL,  -- METADATA, 
  expiration_date TIMESTAMP,  -- METADATA, 
  kind SHA256HASH NOT NULL,
  word SHA256HASH NOT NULL
);
