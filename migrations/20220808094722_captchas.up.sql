CREATE TABLE captchas (
    id text NOT NULL,
    solution text NOT NULL,
    expires timestamp(0) with time zone DEFAULT now() NOT NULL,

    CONSTRAINT captchas_id_check CHECK (length(id) = 40),
    CONSTRAINT captchas_solution_check CHECK (length(solution) = 6)
);
