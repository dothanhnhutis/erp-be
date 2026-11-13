-- ==========================================================
-- audit_logs: lịch sử thay đổi dữ liệu của mọi bảng (trừ bảng trong audit_excluded_tables)
--
-- QUY ƯỚC cho mọi migration có tạo bảng (đặt ở CUỐI file):
--     SELECT sync_updated_at_triggers();
--     SELECT sync_audit_triggers();
-- Bảng không muốn audit:
--     INSERT INTO audit_excluded_tables (table_name, reason) VALUES ('<bảng>', '<lý do>');
-- `erp-seeder migrate` cảnh báo nếu có bảng thiếu trigger.
-- ==========================================================
CREATE TABLE IF NOT EXISTS
    audit_logs (
        id UUID NOT NULL DEFAULT uuidv7 (),
        table_name VARCHAR(100) NOT NULL,
        record_id TEXT NOT NULL,
        action VARCHAR(10) NOT NULL, -- INSERT, UPDATE, DELETE
        old_data JSONB,
        new_data JSONB,
        changed_by TEXT NOT NULL,
        transaction_id TEXT,
        changed_at TIMESTAMPTZ (3) DEFAULT NOW() NOT NULL,
        CONSTRAINT pk_audit_logs PRIMARY KEY (id, changed_at) -- Phải bao gồm cột phân mảnh
    )
PARTITION BY
    RANGE (changed_at);

CREATE INDEX IF NOT EXISTS idx_audit_logs_table_record ON audit_logs (table_name, record_id);

CREATE INDEX IF NOT EXISTS idx_audit_logs_tx ON audit_logs (transaction_id);

-- ==========================================================
-- Danh sách bảng KHÔNG audit (mặc định mọi bảng đều được audit)
-- ==========================================================
CREATE TABLE IF NOT EXISTS
    audit_excluded_tables (
        table_name TEXT NOT NULL,
        reason TEXT NOT NULL,
        created_at TIMESTAMPTZ (3) NOT NULL DEFAULT NOW(),
        CONSTRAINT pk_audit_excluded_tables PRIMARY KEY (table_name)
    );

INSERT INTO
    audit_excluded_tables (table_name, reason)
VALUES
    ('_sqlx_migrations', 'bảng theo dõi migration của sqlx'),
    ('audit_logs', 'chính bảng audit'),
    ('audit_excluded_tables', 'cấu hình audit'),
    (
        'user_sessions',
        'last_seen_at cập nhật mỗi request → audit phình to; có token hash'
    )
ON CONFLICT (table_name) DO NOTHING;

-- ==========================================================
-- Hàm trigger ghi audit (dùng chung cho mọi bảng)
-- Tham số trigger (TG_ARGV) = tên các cột khoá chính, do sync_audit_triggers truyền vào.
-- ==========================================================
CREATE
OR REPLACE FUNCTION fn_generic_audit_log () RETURNS TRIGGER LANGUAGE plpgsql AS $$
DECLARE
    v_tx_id     TEXT;
    v_user_id   TEXT;
    v_record_id TEXT;

    v_old_data  JSONB := NULL;
    v_new_data  JSONB := NULL;

    v_pk_values TEXT[] := '{}';
    v_row_data  JSONB;

    -- cột nhạy cảm cần che trong audit (mask theo TÊN cột, áp dụng cho mọi bảng)
    v_sensitive TEXT[] := ARRAY['password_hash', 'token_hash', 'refresh_token_hash', 'previous_token_hash', 'password', 'secret', 'refresh_token'];
    v_key       TEXT;
BEGIN
    -- 1. Transaction ID (gom các thay đổi cùng 1 transaction)
    v_tx_id := current_setting('ich_app.current_transaction_id', true);
    IF v_tx_id = '' OR v_tx_id IS NULL THEN
        v_tx_id := uuidv7()::TEXT;
        PERFORM set_config('ich_app.current_transaction_id', v_tx_id, true);
    END IF;

    -- 2. User thực hiện (app/seeder đặt bằng set_config('ich_app.current_user_id', ..., true))
    v_user_id := current_setting('ich_app.current_user_id', true);
    IF v_user_id = '' OR v_user_id IS NULL THEN
        v_user_id := 'SYSTEM';
        PERFORM set_config('ich_app.current_user_id', v_user_id, true);
    END IF;

    -- 3. Diff JSONB theo hành động
    IF TG_OP = 'UPDATE' THEN
        SELECT jsonb_object_agg(key, o.value),
               jsonb_object_agg(key, n.value)
        INTO v_old_data, v_new_data
        FROM jsonb_each(to_jsonb(OLD)) o
                 JOIN jsonb_each(to_jsonb(NEW)) n USING (key)
        WHERE o.value IS DISTINCT FROM n.value;

        -- UPDATE nhưng dữ liệu không đổi → không ghi rác
        IF v_new_data IS NULL THEN
            RETURN NULL;
        END IF;
    ELSIF TG_OP = 'INSERT' THEN
        v_new_data := to_jsonb(NEW);
    ELSIF TG_OP = 'DELETE' THEN
        v_old_data := to_jsonb(OLD);
    END IF;

    -- 4. Che dữ liệu nhạy cảm (giữ key để vẫn biết trường đó có thay đổi)
    FOREACH v_key IN ARRAY v_sensitive LOOP
        IF v_old_data ? v_key AND v_old_data ->> v_key IS NOT NULL THEN
            v_old_data := jsonb_set(v_old_data, ARRAY[v_key], '"***REDACTED***"');
        END IF;
        IF v_new_data ? v_key AND v_new_data ->> v_key IS NOT NULL THEN
            v_new_data := jsonb_set(v_new_data, ARRAY[v_key], '"***REDACTED***"');
        END IF;
    END LOOP;

    -- 5. Khoá chính của bản ghi (NEW cho INSERT/UPDATE, OLD cho DELETE)
    v_row_data := CASE WHEN TG_OP = 'DELETE' THEN to_jsonb(OLD) ELSE to_jsonb(NEW) END;

    IF TG_NARGS = 0 THEN
        RAISE EXCEPTION 'trigger audit trên bảng % thiếu tham số khoá chính — gắn lại bằng sync_audit_triggers(true)', TG_TABLE_NAME;
    END IF;

    FOR i IN 0 .. (TG_NARGS - 1) LOOP
        v_pk_values := array_append(v_pk_values, (v_row_data ->> TG_ARGV[i]));
    END LOOP;
    v_record_id := array_to_string(v_pk_values, ':');

    -- 6. Ghi log
    INSERT INTO audit_logs (transaction_id, table_name, record_id, action, old_data, new_data, changed_by)
    VALUES (v_tx_id, TG_TABLE_NAME, v_record_id, TG_OP, v_old_data, v_new_data, v_user_id);

    RETURN NULL;
END;
$$;

-- ==========================================================
-- sync_audit_triggers: gắn trigger audit cho mọi bảng public chưa có và không bị loại trừ;
-- gỡ trigger khỏi bảng vừa bị đưa vào audit_excluded_tables.
-- p_force = true → gắn lại cho mọi bảng (vd sau khi đổi khoá chính của một bảng).
-- Trả về số bảng đã thay đổi.
-- ==========================================================
CREATE
OR REPLACE FUNCTION sync_audit_triggers (p_force BOOLEAN DEFAULT FALSE) RETURNS INTEGER LANGUAGE plpgsql AS $$
DECLARE
    r           RECORD;
    v_trg_insdel TEXT;
    v_trg_upd    TEXT;
    v_trg_count  INTEGER;
    v_pk_args    TEXT;
    v_changed    INTEGER := 0;
BEGIN
    FOR r IN
        SELECT c.oid,
               c.relname,
               EXISTS (SELECT 1 FROM audit_excluded_tables e WHERE e.table_name = c.relname) AS excluded
        FROM pg_class c
                 JOIN pg_namespace n ON n.oid = c.relnamespace
        WHERE n.nspname = 'public'
          AND c.relkind IN ('r', 'p')   -- bảng thường + bảng cha phân vùng
          AND NOT c.relispartition       -- partition tự thừa hưởng trigger của bảng cha
        ORDER BY c.relname
        LOOP
            -- Postgres cắt tên định danh ở 63 byte → tính đúng tên thật
            v_trg_insdel := left('trg_audit_ins_del_' || r.relname, 63);
            v_trg_upd := left('trg_audit_upd_' || r.relname, 63);

            SELECT count(*)
            INTO v_trg_count
            FROM pg_trigger t
            WHERE t.tgrelid = r.oid
              AND t.tgname IN (v_trg_insdel, v_trg_upd);

            -- Bảng bị loại trừ: gỡ trigger nếu còn
            IF r.excluded THEN
                IF v_trg_count > 0 THEN
                    EXECUTE format('DROP TRIGGER IF EXISTS %I ON public.%I', v_trg_insdel, r.relname);
                    EXECUTE format('DROP TRIGGER IF EXISTS %I ON public.%I', v_trg_upd, r.relname);
                    v_changed := v_changed + 1;
                END IF;
                CONTINUE;
            END IF;

            -- Đã đủ trigger → bỏ qua (không khoá lại bảng)
            CONTINUE WHEN v_trg_count = 2 AND NOT p_force;

            -- Cột khoá chính, giữ đúng thứ tự trong index
            SELECT string_agg(quote_literal(a.attname), ', ' ORDER BY array_position(i.indkey::int2[], a.attnum))
            INTO v_pk_args
            FROM pg_index i
                     JOIN pg_attribute a ON a.attrelid = i.indrelid AND a.attnum = ANY (i.indkey)
            WHERE i.indrelid = r.oid
              AND i.indisprimary;

            IF v_pk_args IS NULL THEN
                RAISE EXCEPTION 'bảng "%" không có khoá chính — thêm PRIMARY KEY hoặc INSERT INTO audit_excluded_tables (table_name, reason) VALUES (%, ''...'')',
                    r.relname, quote_literal(r.relname);
            END IF;

            EXECUTE format('DROP TRIGGER IF EXISTS %I ON public.%I', v_trg_insdel, r.relname);
            EXECUTE format('DROP TRIGGER IF EXISTS %I ON public.%I', v_trg_upd, r.relname);

            EXECUTE format(
                    'CREATE TRIGGER %I AFTER INSERT OR DELETE ON public.%I
                     FOR EACH ROW EXECUTE FUNCTION fn_generic_audit_log(%s)',
                    v_trg_insdel, r.relname, v_pk_args);

            EXECUTE format(
                    'CREATE TRIGGER %I AFTER UPDATE ON public.%I
                     FOR EACH ROW
                     WHEN (OLD.* IS DISTINCT FROM NEW.*)
                     EXECUTE FUNCTION fn_generic_audit_log(%s)',
                    v_trg_upd, r.relname, v_pk_args);

            v_changed := v_changed + 1;
        END LOOP;

    RETURN v_changed;
END;
$$;

-- ==========================================================
-- Phân vùng theo tháng bằng pg_partman
-- ==========================================================
CREATE SCHEMA IF NOT EXISTS partman;

-- Cần quyền superuser hoặc owner DB. Ở production nên cài extension trước bằng tài khoản quản trị.
CREATE EXTENSION IF NOT EXISTS pg_partman SCHEMA partman;

-- Bọc idempotent: an toàn khi chạy lại, hoặc khi cài partman SAU.
DO $$
    BEGIN
        IF NOT EXISTS (SELECT 1
                       FROM partman.part_config
                       WHERE parent_table = 'public.audit_logs') THEN
            PERFORM partman.create_parent(
                    p_parent_table := 'public.audit_logs',
                    p_control := 'changed_at',
                    p_interval := '1 month',
                    p_premake := 3 -- số partition tương lai tạo sẵn
                    );
        END IF;
    END
$$;

-- Retention: giữ 1 năm, hết hạn DROP hẳn partition.
-- pg_partman_bgw (bật trong docker-compose) tự chạy maintenance — không cần cron riêng.
UPDATE partman.part_config
SET
    retention = '1 year',
    retention_keep_table = false -- false = DROP partition cũ; true = chỉ detach (vẫn giữ disk)
WHERE
    parent_table = 'public.audit_logs';

-- ==========================================================
-- Gắn trigger cho các bảng hiện có (luôn đặt cuối file)
-- ==========================================================
SELECT sync_updated_at_triggers ();

SELECT sync_audit_triggers ();
