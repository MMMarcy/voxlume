-- Add migration script here
CREATE TABLE IF NOT EXISTS users (
    id BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    username TEXT UNIQUE NOT NULL,
    anonymous BOOLEAN NOT NULL,
    password_mcf TEXT NOT NULL,
    last_access TIMESTAMP WITH TIME ZONE NOT NULL
);

INSERT INTO users (username, anonymous, password_mcf, last_access)
VALUES (
    'guest',
    false,
    '',
    NOW()
)
ON CONFLICT (username) DO NOTHING;
