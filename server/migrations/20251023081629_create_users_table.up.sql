-- Add up migration script here

CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

INSERT INTO "users" ( "id", "username", "password_hash") VALUES ( 1, 'admin', '$2b$12$/MZyRsK.DcYHh6x4qCy6IOjxO/Wd4RlPSbW.7OiAYqTY4U4CipDIS');
