CREATE TABLE users (
    id integer NOT NULL,
    name text NOT NULL,
    role text NOT NULL,
    password text NOT NULL,
    created timestamp(0) with time zone DEFAULT now() NOT NULL,
    CONSTRAINT users_name_check CHECK (length(name) < 64)
);
