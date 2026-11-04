-- Execute statements individually with autocommit, outside a transaction.
CREATE EXTENSION IF NOT EXISTS pg_trgm WITH SCHEMA public;

CREATE INDEX CONCURRENTLY IF NOT EXISTS article_search_trgm_idx
ON public.article USING gin (
    (lower(article_title || ' ' || coalesce(prospectus, '') || ' ' || coalesce(article_content, ''))) public.gin_trgm_ops
)
WHERE deleted = 0;

CREATE INDEX CONCURRENTLY IF NOT EXISTS article_active_update_id_idx
ON public.article (update_time DESC NULLS LAST, id DESC)
WHERE deleted = 0;
