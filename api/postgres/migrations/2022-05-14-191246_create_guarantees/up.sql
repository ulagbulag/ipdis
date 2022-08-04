-- Your SQL goes here
CREATE TABLE accounts_guarantees (
  id SERIAL PRIMARY KEY,
  -- METADATA BEGIN --
  nonce NONCE NOT NULL,
  guarantee ACCOUNT NOT NULL,
  guarantor ACCOUNT NOT NULL,
  guarantee_signature SIGNATURE NOT NULL UNIQUE,
  guarantor_signature SIGNATURE NOT NULL UNIQUE,
  created_date TIMESTAMP NOT NULL,
  expiration_date TIMESTAMP,
  hash SHA256HASH NOT NULL -- METADATA END --
);