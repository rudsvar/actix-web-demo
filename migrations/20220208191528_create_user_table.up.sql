CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    password TEXT NOT NULL,
    created_at timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP
);
SELECT setval('users_id_seq', 999);

INSERT INTO users (id, name, password) VALUES (1, 'user', '$2a$10$2S5cSYETYr7vkh.ucqdBbeiwxm58WtSYZshHICnCGfp/PBjs3cDS6');
INSERT INTO users (id, name, password) VALUES (2, 'admin', '$2a$10$QQOlwlV1/FyE2QbZZy4Bs.sMk1/9as3ZALQvCtcEwuMV5tP/guA3u');
