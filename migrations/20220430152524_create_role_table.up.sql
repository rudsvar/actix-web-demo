CREATE TYPE ROLE_NAME AS ENUM ('User', 'Admin');

CREATE TABLE role (
    id SERIAL PRIMARY KEY,
    name ROLE_NAME NOT NULL
);
SELECT setval('role_id_seq', 999);

INSERT INTO role VALUES (1, 'User');
INSERT INTO role VALUES (2, 'Admin');

CREATE TABLE user_role (
    id SERIAL PRIMARY KEY,
    user_id INT NOT NULL,
    role_id INT NOT NULL,

    CONSTRAINT fk_user_id
        FOREIGN KEY (user_id)
        REFERENCES users(id),

    CONSTRAINT fk_role_id
        FOREIGN KEY (role_id)
        REFERENCES role(id)
);
SELECT setval('user_role_id_seq', 999);

INSERT INTO user_role VALUES (1, 1, 1);
INSERT INTO user_role VALUES (2, 2, 2);
