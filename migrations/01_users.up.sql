CREATE TABLE users (
    id serial NOT NULL,
    name text UNIQUE NOT NULL,
    role text NOT NULL,
    password text NOT NULL,
    created timestamp(0) with time zone DEFAULT now() NOT NULL,
    CONSTRAINT users_name_check CHECK (length(name) < 64)
);
