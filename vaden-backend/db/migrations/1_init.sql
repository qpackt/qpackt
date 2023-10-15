CREATE TABLE versions (
    id INT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    path TEXT NOT NULL,
    created TIMESTAMP NOT NULL
);

CREATE TABLE settings (
    name TEXT NOT NULL UNIQUE,
    value JSON NOT NULL
)