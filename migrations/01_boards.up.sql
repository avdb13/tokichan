CREATE TABLE boards (
    name text NOT NULL,
    title text NOT NULL,
    CONSTRAINT boards_name_check CHECK (length(name) = 1),
    CONSTRAINT boards_title_check CHECK (length(title) < 64)
);
