CREATE TABLE posts (
    id integer NOT NULL,
    parent integer,
    board text NOT NULL,
    op text DEFAULT 'Anonymous'::text NOT NULL,
    email text,
    subject text,
    body text,
    created timestamp(0) with time zone DEFAULT now() NOT NULL,

    files text [],

    CONSTRAINT posts_body_check CHECK (length(body) < 32768),
    CONSTRAINT posts_email_check CHECK (length(email) < 64),
    CONSTRAINT posts_op_check CHECK (length(op) < 64),
    CONSTRAINT posts_subject_check CHECK (length(subject) < 64)
);
