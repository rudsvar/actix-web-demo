CREATE TABLE accounts (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    balance BIGINT NOT NULL,
    owner_id INT NOT NULL REFERENCES users(id)
);
SELECT setval('accounts_id_seq', 999);

CREATE TABLE transfers (
    id SERIAL PRIMARY KEY,
    from_account INT NOT NULL REFERENCES accounts(id),
    to_account INT NOT NULL REFERENCES accounts(id),
    amount BIGINT NOT NULL,
    created_at timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP
);
SELECT setval('transfers_id_seq', 999);

INSERT INTO accounts (id, name, balance, owner_id) VALUES (1, 'acc1', 100, 1);
INSERT INTO accounts (id, name, balance, owner_id) VALUES (2, 'acc2', 500, 1);
INSERT INTO accounts (id, name, balance, owner_id) VALUES (3, 'acc3', 700, 2);

INSERT INTO transfers (id, from_account, to_account, amount) VALUES (1, 1, 2, 200);
INSERT INTO transfers (id, from_account, to_account, amount) VALUES (2, 2, 3, 100);
