CREATE TABLE IF NOT EXISTS public.hardcover_audiobook_metadata (
id BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
audiobook_id BIGINT NOT NULL UNIQUE,
metadata JSONB NOT NULL,
CONSTRAINT fk_hardcover_audiobook FOREIGN KEY (audiobook_id)
REFERENCES public.audiobook (id) ON DELETE CASCADE
) ;

CREATE TABLE IF NOT EXISTS public.hardcover_author_metadata (
id BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
author_id BIGINT NOT NULL UNIQUE,
metadata JSONB NOT NULL,
CONSTRAINT fk_hardcover_author FOREIGN KEY (author_id)
REFERENCES public.author (id) ON DELETE CASCADE
) ;

CREATE TABLE IF NOT EXISTS public.hardcover_series_metadata (
id BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
series_id BIGINT NOT NULL UNIQUE,
metadata JSONB NOT NULL,
CONSTRAINT fk_hardcover_series FOREIGN KEY (series_id)
REFERENCES public.series (id) ON DELETE CASCADE
) ;
