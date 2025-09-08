CREATE TABLE users
(
    id         SERIAL PRIMARY KEY,
    username   TEXT NOT NULL UNIQUE,
    email      TEXT NOT NULL UNIQUE,
    created_at TIMESTAMP DEFAULT NOW()
);