-- Copy the legacy runtime tables without changing existing public.users/user_articles.
-- Run once; existing target tables cause a rollback rather than an overwrite.
BEGIN;
SET LOCAL lock_timeout = '5s';
SET LOCAL statement_timeout = '60s';
LOCK TABLE poetize."user", poetize.article, poetize.key_pair, poetize.user_session IN SHARE MODE;
DO $$
DECLARE
    tbl text;
    col record;
    seq record;
    fk record;
    newseq text;
    differences bigint;
BEGIN
    FOREACH tbl IN ARRAY ARRAY['user', 'article', 'key_pair', 'user_session'] LOOP
        IF to_regclass(format('public.%I', tbl)) IS NOT NULL THEN
            RAISE EXCEPTION 'Target public.% already exists; refusing to overwrite', tbl;
        END IF;
        EXECUTE format('CREATE TABLE public.%I (LIKE poetize.%I INCLUDING ALL)', tbl, tbl);
        FOR col IN
            SELECT column_name, pg_get_serial_sequence(format('poetize.%I', tbl), column_name) AS source_sequence
            FROM information_schema.columns
            WHERE table_schema = 'poetize' AND table_name = tbl
        LOOP
            IF col.source_sequence IS NULL THEN CONTINUE; END IF;
            SELECT *, format_type(seqtypid, NULL) AS type_name INTO seq
                FROM pg_sequence WHERE seqrelid = col.source_sequence::regclass;
            newseq := tbl || '_' || col.column_name || '_seq';
            EXECUTE format('CREATE SEQUENCE public.%I AS %s INCREMENT BY %s MINVALUE %s MAXVALUE %s START WITH %s CACHE %s %s',
                newseq, seq.type_name, seq.seqincrement, seq.seqmin, seq.seqmax, seq.seqstart, seq.seqcache,
                CASE WHEN seq.seqcycle THEN 'CYCLE' ELSE 'NO CYCLE' END);
            EXECUTE format('ALTER SEQUENCE public.%I OWNED BY public.%I.%I', newseq, tbl, col.column_name);
            EXECUTE format('ALTER TABLE public.%I ALTER COLUMN %I SET DEFAULT nextval(%L::regclass)', tbl, col.column_name, format('public.%I', newseq));
        END LOOP;
        EXECUTE format('INSERT INTO public.%I SELECT * FROM poetize.%I', tbl, tbl);
        FOR col IN
            SELECT column_name, pg_get_serial_sequence(format('poetize.%I', tbl), column_name) AS source_sequence
            FROM information_schema.columns WHERE table_schema = 'poetize' AND table_name = tbl
        LOOP
            IF col.source_sequence IS NULL THEN CONTINUE; END IF;
            newseq := tbl || '_' || col.column_name || '_seq';
            EXECUTE format('SELECT setval(%L::regclass, GREATEST(s.last_value, COALESCE((SELECT max(%I)::bigint FROM public.%I), s.last_value)), s.is_called OR EXISTS(SELECT 1 FROM public.%I)) FROM %s s',
                format('public.%I', newseq), col.column_name, tbl, tbl, col.source_sequence);
            IF pg_get_serial_sequence(format('public.%I', tbl), col.column_name)::regclass::oid = col.source_sequence::regclass::oid THEN
                RAISE EXCEPTION 'Sequence still shared for %.%', tbl, col.column_name;
            END IF;
        END LOOP;
        EXECUTE format('SELECT count(*) FROM ((SELECT * FROM poetize.%I EXCEPT ALL SELECT * FROM public.%I) UNION ALL (SELECT * FROM public.%I EXCEPT ALL SELECT * FROM poetize.%I)) d', tbl, tbl, tbl, tbl) INTO differences;
        IF differences <> 0 THEN RAISE EXCEPTION 'Data mismatch for %', tbl; END IF;
    END LOOP;
    FOR fk IN
        SELECT c.conname, cl.relname, pg_get_constraintdef(c.oid) AS definition
        FROM pg_constraint c JOIN pg_class cl ON cl.oid=c.conrelid JOIN pg_namespace n ON n.oid=cl.relnamespace
        WHERE n.nspname='poetize' AND cl.relname IN ('user','article','key_pair','user_session') AND c.contype='f'
    LOOP
        EXECUTE format('ALTER TABLE public.%I ADD CONSTRAINT %I %s', fk.relname, fk.conname, replace(fk.definition, 'poetize.', 'public.'));
    END LOOP;
END $$;
COMMIT;
