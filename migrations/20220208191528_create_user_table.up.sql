CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    password TEXT NOT NULL UNIQUE,
    created_at timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP
);
SELECT setval('users_id_seq', 999);

INSERT INTO users (id, name, password) VALUES (1, 'foo', 'abc');
INSERT INTO users (id, name, password) VALUES (2, 'bar', '123');
