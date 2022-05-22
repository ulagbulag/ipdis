-- Your SQL goes here
CREATE TABLE words (
  id SERIAL PRIMARY KEY,
  -- METADATA BEGIN --
  nonce NONCE NOT NULL,
  guarantee ACCOUNT NOT NULL,
  guarantor ACCOUNT NOT NULL,
  guarantee_signature SIGNATURE NOT NULL UNIQUE,
  guarantor_signature SIGNATURE NOT NULL UNIQUE,
  created_date TIMESTAMP NOT NULL,
  expiration_date TIMESTAMP,
  -- METADATA END --
  kind SHA256HASH NOT NULL,
  parent SHA256HASH NOT NULL,
  lang SHA256HASH NOT NULL,
  word SHA256HASH NOT NULL
);
CREATE TABLE words_counts (
  id SERIAL PRIMARY KEY,
  kind SHA256HASH NOT NULL,
  parent SHA256HASH NOT NULL,
  lang SHA256HASH NOT NULL,
  word SHA256HASH NOT NULL,
  count BIGINT NOT NULL,
  UNIQUE (kind, lang, word)
);
CREATE TABLE words_counts_guarantees (
  id SERIAL PRIMARY KEY,
  guarantee ACCOUNT NOT NULL,
  kind SHA256HASH NOT NULL,
  parent SHA256HASH NOT NULL,
  lang SHA256HASH NOT NULL,
  word SHA256HASH NOT NULL,
  count BIGINT NOT NULL,
  UNIQUE (guarantee, kind, lang, word)
);