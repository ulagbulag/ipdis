-- Your SQL goes here
CREATE TABLE dynpaths (
  id SERIAL PRIMARY KEY,
  account VARCHAR(44) NOT NULL,  -- METADATA, ED25519 PublicKey - base58 = 32 bytes
  signature VARCHAR(88) NOT NULL,  -- METADATA, ED25519 Signature - base58 = 64 bytes
  created_date DATETIME NOT NULL,  -- METADATA, 
  expiration_date DATETIME NOT NULL,  -- METADATA, 
  name VARCHAR NOT NULL,
  path VARCHAR(44) NOT NULL,  -- SHA256 - base58 = 32 bytes
  len BIGINT NOT NULL
)
