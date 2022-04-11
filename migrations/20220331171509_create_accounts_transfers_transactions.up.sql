CREATE TABLE accounts (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    balance INTEGER NOT NULL,
    owner_id INTEGER NOT NULL,

    CONSTRAINT fk_owner_id
        FOREIGN KEY (owner_id)
        REFERENCES users(id)
);
SELECT setval('accounts_id_seq', 999);

CREATE TABLE transfers (
    id SERIAL PRIMARY KEY,
    from_account INTEGER NOT NULL,
    to_account INTEGER NOT NULL,
    amount INTEGER NOT NULL,
    created_at timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT fk_from_account
        FOREIGN KEY (from_account)
        REFERENCES accounts(id),

    CONSTRAINT fk_to_account
        FOREIGN KEY (to_account)
        REFERENCES accounts(id)
);
SELECT setval('transfers_id_seq', 999);

INSERT INTO accounts (id, name, balance, owner_id) VALUES (1, 'acc1',   0, 1);
INSERT INTO accounts (id, name, balance, owner_id) VALUES (2, 'acc2', 500, 1);
INSERT INTO accounts (id, name, balance, owner_id) VALUES (3, 'acc3', 700, 2);

INSERT INTO transfers (id, from_account, to_account, amount) VALUES (1, 1, 2, 200);
INSERT INTO transfers (id, from_account, to_account, amount) VALUES (2, 2, 3, 100);
