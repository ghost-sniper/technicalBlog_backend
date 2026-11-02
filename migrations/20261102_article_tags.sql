BEGIN;
SET LOCAL lock_timeout='5s';
CREATE TABLE IF NOT EXISTS public.article_tags (
    article_id INTEGER NOT NULL REFERENCES public.article(id) ON DELETE CASCADE,
    tag_id BIGINT NOT NULL REFERENCES public.tag(id) ON DELETE RESTRICT,
    PRIMARY KEY(article_id,tag_id)
);
CREATE INDEX IF NOT EXISTS article_tags_tag_id_idx ON public.article_tags(tag_id);
COMMIT;
