-- =================================================================
-- STEP 1: Create all tables in the 'public' schema
-- =================================================================

CREATE TABLE IF NOT EXISTS public.users (
id BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
username TEXT UNIQUE NOT NULL,
anonymous BOOLEAN NOT NULL,
password_mcf TEXT NOT NULL,
last_access TIMESTAMP WITH TIME ZONE NOT NULL
) ;

CREATE TABLE IF NOT EXISTS public.series (
id BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
title TEXT NOT NULL UNIQUE
) ;

CREATE TABLE IF NOT EXISTS public.keyword (
id BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
name TEXT NOT NULL UNIQUE
) ;

CREATE TABLE IF NOT EXISTS public.category (
id BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
name TEXT NOT NULL UNIQUE
) ;

CREATE TABLE IF NOT EXISTS public.author (
id BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
name TEXT NOT NULL UNIQUE
) ;

CREATE TABLE IF NOT EXISTS public.reader (
id BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
name TEXT NOT NULL UNIQUE
) ;

CREATE TABLE IF NOT EXISTS public.audiobook (
id BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
series_id BIGINT NULL,
title TEXT NOT NULL,
bitrate INTEGER NULL,
cover_url TEXT NOT NULL,
description TEXT NOT NULL,
description_for_embeddings TEXT NOT NULL,
file_size BIGINT NULL,
format TEXT NOT NULL,
language TEXT NOT NULL,
path TEXT NOT NULL UNIQUE,
timestamp_created TIMESTAMP WITH TIME ZONE NOT NULL,
timestamp_ingested TIMESTAMP WITH TIME ZONE NOT NULL,
unabridged BOOLEAN NULL,
very_short_description TEXT NOT NULL,
optimized_description_embedding VECTOR (768),
CONSTRAINT fk_series FOREIGN KEY (series_id) REFERENCES public.series (id)
) ;

CREATE TABLE IF NOT EXISTS public.audiobook_author (
audiobook_id BIGINT NOT NULL,
author_id BIGINT NOT NULL,
PRIMARY KEY (audiobook_id, author_id),
FOREIGN KEY (audiobook_id) REFERENCES public.audiobook (id) ON DELETE CASCADE,
FOREIGN KEY (author_id) REFERENCES public.author (id) ON DELETE CASCADE
) ;

CREATE TABLE IF NOT EXISTS public.audiobook_reader (
audiobook_id BIGINT NOT NULL,
reader_id BIGINT NOT NULL,
PRIMARY KEY (audiobook_id, reader_id),
FOREIGN KEY (audiobook_id) REFERENCES public.audiobook (id) ON DELETE CASCADE,
FOREIGN KEY (reader_id) REFERENCES public.reader (id) ON DELETE CASCADE
) ;

CREATE TABLE IF NOT EXISTS public.audiobook_keyword (
audiobook_id BIGINT NOT NULL,
keyword_id BIGINT NOT NULL,
PRIMARY KEY (audiobook_id, keyword_id),
FOREIGN KEY (audiobook_id) REFERENCES public.audiobook (id) ON DELETE CASCADE,
FOREIGN KEY (keyword_id) REFERENCES public.keyword (id) ON DELETE CASCADE
) ;

CREATE TABLE IF NOT EXISTS public.audiobook_category (
audiobook_id BIGINT NOT NULL,
category_id BIGINT NOT NULL,
PRIMARY KEY (audiobook_id, category_id),
FOREIGN KEY (audiobook_id) REFERENCES public.audiobook (id) ON DELETE CASCADE,
FOREIGN KEY (category_id) REFERENCES public.category (id) ON DELETE CASCADE
) ;

CREATE TABLE IF NOT EXISTS public.user_series_notification (
user_id BIGINT NOT NULL,
series_id BIGINT NOT NULL,
PRIMARY KEY (user_id, series_id),
FOREIGN KEY (user_id) REFERENCES public.users (id) ON DELETE CASCADE,
FOREIGN KEY (series_id) REFERENCES public.series (id) ON DELETE CASCADE
) ;

CREATE TABLE IF NOT EXISTS public.user_keyword_notification (
user_id BIGINT NOT NULL,
keyword_id BIGINT NOT NULL,
PRIMARY KEY (user_id, keyword_id),
FOREIGN KEY (user_id) REFERENCES public.users (id) ON DELETE CASCADE,
FOREIGN KEY (keyword_id) REFERENCES public.keyword (id) ON DELETE CASCADE
) ;

CREATE TABLE IF NOT EXISTS public.user_category_notification (
user_id BIGINT NOT NULL,
category_id BIGINT NOT NULL,
PRIMARY KEY (user_id, category_id),
FOREIGN KEY (user_id) REFERENCES public.users (id) ON DELETE CASCADE,
FOREIGN KEY (category_id) REFERENCES public.category (id) ON DELETE CASCADE
) ;

CREATE TABLE IF NOT EXISTS public.user_author_notification (
user_id BIGINT NOT NULL,
author_id BIGINT NOT NULL,
PRIMARY KEY (user_id, author_id),
FOREIGN KEY (user_id) REFERENCES public.users (id) ON DELETE CASCADE,
FOREIGN KEY (author_id) REFERENCES public.author (id) ON DELETE CASCADE
) ;

CREATE TABLE IF NOT EXISTS public.user_reader_notification (
user_id BIGINT NOT NULL,
reader_id BIGINT NOT NULL,
PRIMARY KEY (user_id, reader_id),
FOREIGN KEY (user_id) REFERENCES public.users (id) ON DELETE CASCADE,
FOREIGN KEY (reader_id) REFERENCES public.reader (id) ON DELETE CASCADE
) ;

DO $$
BEGIN
    -- Check if the type 'notification_reason' exists in the 'public' schema
    IF NOT EXISTS (
        SELECT 1 
        FROM pg_type t
        JOIN pg_namespace n ON n.oid = t.typnamespace
        WHERE t.typname = 'notification_reason' AND n.nspname = 'public'
    ) THEN
        -- If it doesn't exist, create it
        CREATE TYPE public.notification_reason AS ENUM (
            'match_series',
            'match_keyword',
            'match_category',
            'match_author',
            'match_reader'
        );
    END IF;
END$$ ;

CREATE TABLE IF NOT EXISTS public.user_notification (
user_id BIGINT NOT NULL,
audiobook_id BIGINT NOT NULL,
created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
reasons notification_reason [] DEFAULT '{}',
has_been_seen BOOLEAN DEFAULT FALSE,
PRIMARY KEY (user_id, audiobook_id),
FOREIGN KEY (user_id) REFERENCES public.users (id) ON DELETE CASCADE,
FOREIGN KEY (audiobook_id) REFERENCES public.audiobook (id) ON DELETE CASCADE
) ;


CREATE TABLE IF NOT EXISTS public.user_favorite_audiobook (
user_id BIGINT NOT NULL,
audiobook_id BIGINT NOT NULL,
PRIMARY KEY (user_id, audiobook_id),
FOREIGN KEY (user_id) REFERENCES public.users (id) ON DELETE CASCADE,
FOREIGN KEY (audiobook_id) REFERENCES public.audiobook (id) ON DELETE CASCADE
) ;

-- =================================================================
-- STEP 2: Create the search function in the 'public' schema
-- =================================================================

CREATE MATERIALIZED VIEW public.audiobook_search_view AS
SELECT
a.id AS audiobook_id,
-- Combine all text into one column
CONCAT_WS (' ',
a.title,
a.very_short_description,
a.description,
(SELECT string_agg (au.name, ' ')
FROM public.audiobook_author aba
JOIN public.author au ON au.id = aba.author_id
WHERE aba.audiobook_id = a.id),
(SELECT string_agg (r.name, ' ')
FROM public.audiobook_reader abr
JOIN public.reader r ON r.id = abr.reader_id
WHERE abr.audiobook_id = a.id),
(SELECT string_agg (k.name, ' ')
FROM public.audiobook_keyword abk
JOIN public.keyword k ON k.id = abk.keyword_id
WHERE abk.audiobook_id = a.id),
(SELECT string_agg (c.name, ' ')
FROM public.audiobook_category abc
JOIN public.category c ON c.id = abc.category_id
WHERE abc.audiobook_id = a.id)
) AS search_content
FROM public.audiobook a ;


-- =================================================================
-- STEP 3: Create all indexes on tables in the 'public' schema
-- =================================================================

-- Standard indexes for performance
CREATE INDEX IF NOT EXISTS idx_audiobook_author_author_id ON
public.audiobook_author (author_id) ;
CREATE INDEX IF NOT EXISTS idx_audiobook_reader_reader_id ON
public.audiobook_reader (reader_id) ;
CREATE INDEX IF NOT EXISTS idx_audiobook_keyword_keyword_id ON
public.audiobook_keyword (keyword_id) ;
CREATE INDEX IF NOT EXISTS idx_audiobook_category_category_id ON
public.audiobook_category (category_id) ;
CREATE INDEX IF NOT EXISTS idx_user_series_notification_series_id ON
public.user_series_notification (series_id) ;
CREATE INDEX IF NOT EXISTS idx_user_keyword_notification_keyword_id ON
public.user_keyword_notification (keyword_id) ;
CREATE INDEX IF NOT EXISTS idx_user_category_notification_category_id ON
public.user_category_notification (category_id) ;
CREATE INDEX IF NOT EXISTS idx_user_author_notification_author_id ON
public.user_author_notification (author_id) ;
CREATE INDEX IF NOT EXISTS idx_user_reader_notification_reader_id ON
public.user_reader_notification (reader_id) ;
CREATE INDEX IF NOT EXISTS idx_user_notification_user_id ON
public.user_notification (user_id) ;
CREATE INDEX IF NOT EXISTS idx_user_notification_audiobook_id ON
public.user_notification (audiobook_id) ;
CREATE INDEX IF NOT EXISTS idx_user_favorite_audiobook_audiobook_id ON
public.user_favorite_audiobook (audiobook_id) ;
CREATE UNIQUE INDEX idx_audiobook_search_view_unique_id
ON public.audiobook_search_view (audiobook_id) ;

-- ParadeDB HNSW Index
CREATE INDEX IF NOT EXISTS
optimized_description_embedding_idx ON public.audiobook
USING hnsw (optimized_description_embedding vector_cosine_ops)
WITH (m = 16, ef_construction = 64) ;

-- ParadeDB BM25 Index (references the schema-qualified function)
CREATE INDEX search_idx
ON public.audiobook_search_view
USING bm25 (audiobook_id, search_content)
WITH (key_field = audiobook_id) ;
