CREATE TABLE requests (
    id SERIAL PRIMARY KEY,
    user_id INT REFERENCES users(id),
    ip TEXT NOT NULL,
    request_target TEXT,
    request_body TEXT,
    request_time TIMESTAMP WITH TIME ZONE NOT NULL,
    response_body TEXT,
    response_code INT NOT NULL,
    response_time_ms INT NOT NULL
);
