-- Your SQL goes here
CREATE DOMAIN NONCE as UUID NOT NULL;
-- ED25519 PublicKey - base58 = 32 bytes
CREATE DOMAIN ACCOUNT as VARCHAR(44) NOT NULL CHECK (CHAR_LENGTH(value) >= 43);
-- ED25519 Signature - base58 = 64 bytes
CREATE DOMAIN SIGNATURE as VARCHAR(88) NOT NULL CHECK (CHAR_LENGTH(value) >= 87);
-- SHA256 CID - base32 = 60 bytes
CREATE DOMAIN SHA256HASH as VARCHAR(60) NOT NULL CHECK (CHAR_LENGTH(value) >= 59);