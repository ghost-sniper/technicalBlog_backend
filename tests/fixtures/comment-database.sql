-- Minimal schema for comment/access integration tests in an EMPTY disposable database.
-- Do not execute against an application database. No real user/article data is needed.
CREATE TABLE public."user" (
 id serial PRIMARY KEY, username varchar(32) NOT NULL, password text NOT NULL,
 user_type smallint NOT NULL DEFAULT 1, user_status smallint NOT NULL DEFAULT 1,
 deleted smallint NOT NULL DEFAULT 0, avatar text
);
CREATE TABLE public.article (
 id serial PRIMARY KEY, user_id integer NOT NULL, sort_id integer NOT NULL,
 label_id integer NOT NULL, article_title text NOT NULL, article_content text NOT NULL,
 view_type text NOT NULL DEFAULT 'public', view_value text,
 comment_status smallint NOT NULL DEFAULT 1, deleted smallint NOT NULL DEFAULT 0,
 prospectus text, update_time timestamp DEFAULT clock_timestamp(), update_by text
);
CREATE TABLE public.tag (
 id bigserial PRIMARY KEY, tag_name text NOT NULL, status integer NOT NULL DEFAULT 1
);
