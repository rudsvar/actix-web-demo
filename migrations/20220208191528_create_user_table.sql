-- Add migration script here
CREATE TABLE users (
    id uuid NOT NULL PRIMARY KEY,
    name TEXT NOT NULL,
    password TEXT NOT NULL UNIQUE,
    created_at timestamptz NOT NULL
)
