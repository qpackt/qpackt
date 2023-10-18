CREATE TABLE versions (
    web_root TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL UNIQUE
);

CREATE TABLE settings (
    name TEXT NOT NULL UNIQUE,
    value JSON NOT NULL
)