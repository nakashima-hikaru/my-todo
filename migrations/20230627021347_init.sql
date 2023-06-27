CREATE TABLE todos
(
    id        SERIAL PRIMARY KEY,
    text      TEXT    NOT NULL,
    completed BOOLEAN NOT NULL DEFAULT false
);
CREATE TABLE users
(
    id            SERIAL PRIMARY KEY,
    username      VARCHAR(255) NOT NULL,
    email         VARCHAR(255) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL
);