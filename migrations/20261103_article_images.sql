CREATE TABLE IF NOT EXISTS public.article_images (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    article_id INTEGER NOT NULL REFERENCES public.article(id) ON DELETE CASCADE,
    content_type TEXT NOT NULL CHECK (content_type IN ('image/png', 'image/jpeg', 'image/gif', 'image/webp')),
    content BYTEA NOT NULL CHECK (octet_length(content) BETWEEN 1 AND 5242880)
);
CREATE INDEX IF NOT EXISTS article_images_article_id_idx ON public.article_images(article_id);
