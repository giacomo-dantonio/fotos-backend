-- Add migration script here

CREATE TABLE IF NOT EXISTS files
(
    id              GUID            PRIMARY KEY NOT NULL,
    relative_path   VARCHAR(500)    NOT NULL,
    csum            VARCHAR(500)    NOT NULL
);
