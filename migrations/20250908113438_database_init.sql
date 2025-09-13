CREATE TABLE IF NOT EXISTS authors
(
    id         SERIAL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS posts
(
    id         SERIAL PRIMARY KEY,
    author_id  INTEGER REFERENCES authors (id) ON DELETE CASCADE,
    title      TEXT NOT NULL,
    body       TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT NOW()
);
