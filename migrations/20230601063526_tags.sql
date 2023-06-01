-- Add migration script here

CREATE TABLE IF NOT EXISTS tags
(
    id              GUID            PRIMARY KEY NOT NULL,
    tagname         VARCHAR(500)    NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS filetags
(
    tag_id          GUID            NOT NULL REFERENCES tags(id),
    file_id         GUID            NOT NULL REFERENCES files(id),
    PRIMARY KEY (tag_id, file_id)
);
