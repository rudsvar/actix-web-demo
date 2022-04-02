CREATE TABLE accounts (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    balance INTEGER NOT NULL
);

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

CREATE TABLE transactions (
    id SERIAL PRIMARY KEY,
    account INTEGER NOT NULL,
    amount INTEGER NOT NULL,
    created_at timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT fk_account
        FOREIGN KEY (account)
        REFERENCES accounts(id)
);
