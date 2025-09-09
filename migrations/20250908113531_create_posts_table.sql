CREATE TABLE posts
(
    id         SERIAL PRIMARY KEY,
    author_id INTEGER REFERENCES authors (id) ON DELETE CASCADE,
    title      TEXT NOT NULL,
    body       TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT NOW()
);