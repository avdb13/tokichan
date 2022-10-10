CREATE TABLE files (
    id integer NOT NULL,
    name text NOT NULL,

    CONSTRAINT files_name_check CHECK (length(name) > 20)
);
