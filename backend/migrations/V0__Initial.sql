CREATE TABLE account (
    id uuid PRIMARY KEY,
    email varchar(128) NOT NULL,
    -- Password, hashed with argon2id
    hashed_password bytea NOT NULL,
    name varchar(128) NOT NULL
);

CREATE TABLE user_token (
    id SERIAL PRIMARY KEY,

    token bytea NOT NULL,
    user_id uuid NOT NULL
);
