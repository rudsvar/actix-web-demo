CREATE TABLE audit_log (
    id SERIAL PRIMARY KEY,
    user_id INT NOT NULL REFERENCES users(id),
    module TEXT NOT NULL,
    function TEXT NOT NULL,
    entity_id INT,
    input TEXT,
    output TEXT
);
