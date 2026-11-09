BEGIN;
SET LOCAL lock_timeout = '5s';
CREATE OR REPLACE FUNCTION public.can_read_article(a public.article, viewer integer)
RETURNS boolean LANGUAGE sql STABLE AS $$
 SELECT a.deleted = 0 AND (a.view_type = 'public' OR EXISTS (
   SELECT 1 FROM public."user" u WHERE u.id = viewer AND u.user_status = 1 AND u.deleted = 0
   AND (u.id = a.user_id OR u.user_type = 0 OR a.view_type = 'authenticated')
 ));
$$;
CREATE TABLE public.article_comments (
 id bigserial PRIMARY KEY,
 article_id integer NOT NULL REFERENCES public.article(id),
 user_id integer NOT NULL REFERENCES public."user"(id),
 root_id bigint REFERENCES public.article_comments(id),
 reply_to_comment_id bigint REFERENCES public.article_comments(id),
 reply_to_user_id integer REFERENCES public."user"(id),
 reply_to_username text,
 content text NOT NULL CHECK (char_length(btrim(content)) BETWEEN 1 AND 2000),
 created_at timestamp NOT NULL DEFAULT clock_timestamp(),
 deleted boolean NOT NULL DEFAULT false,
 CHECK ((root_id IS NULL AND reply_to_comment_id IS NULL AND reply_to_user_id IS NULL)
     OR (root_id IS NOT NULL AND reply_to_comment_id IS NOT NULL AND reply_to_user_id IS NOT NULL)),
 CHECK (root_id IS NULL OR root_id <> id)
);
CREATE INDEX article_comments_roots ON public.article_comments(article_id, created_at DESC, id DESC) WHERE root_id IS NULL AND NOT deleted;
CREATE INDEX article_comments_replies ON public.article_comments(article_id, root_id, id) WHERE root_id IS NOT NULL AND NOT deleted;
ALTER TABLE public.article ALTER COLUMN comment_status SET DEFAULT 1;
COMMIT;
