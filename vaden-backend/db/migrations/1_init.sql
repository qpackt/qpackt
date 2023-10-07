CREATE TABLE images (
    id INT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    file TEXT NOT NULL,
    created TIMESTAMP NOT NULL
);