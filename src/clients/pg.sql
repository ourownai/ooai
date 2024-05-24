DROP TABLE user_key;
CREATE TABLE IF NOT EXISTS user_key (
    id             SERIAL PRIMARY KEY,
    user_id        bigint NOT NULL DEFAULT 0,
    key_id         VARCHAR(256) NOT NULL,
    create_time    TIMESTAMP(0) NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE UNIQUE INDEX idx_user_id ON user_key(user_id);

DROP TABLE user_secret;
CREATE TABLE user_secret (
    id             SERIAL PRIMARY KEY,
    key_id         VARCHAR(128) NOT NULL,
    secret         VARCHAR(256) NOT NULL,
    create_time    TIMESTAMP NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE UNIQUE INDEX idx_key_id ON user_secret(key_id);

DROP TABLE jwk;
CREATE TABLE jwk (
    id             SERIAL PRIMARY KEY,
    key_id         VARCHAR(128) NOT NULL,
    jwk            TEXT NOT NULL,
    create_time    TIMESTAMP NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE UNIQUE INDEX idx_key_id ON jwk(key_id);
