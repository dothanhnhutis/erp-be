-- 1. Gỡ cấu hình pg_partman cho audit_logs
DO $$
    BEGIN
        IF to_regclass('partman.part_config') IS NOT NULL THEN
            DELETE FROM partman.part_config WHERE parent_table = 'public.audit_logs';
        END IF;
    END
$$;

-- bảng template do partman.create_parent tạo ra (không thuộc extension nên phải xoá tay)
DROP TABLE IF EXISTS partman.template_public_audit_logs;

-- 2. Hàm audit + MỌI trigger đang dùng nó (CASCADE xoá trigger trên tất cả bảng).
DROP FUNCTION IF EXISTS sync_audit_triggers (BOOLEAN);

DROP FUNCTION IF EXISTS fn_generic_audit_log () CASCADE;

-- 3. Bảng (partition của audit_logs bị xoá theo bảng cha)
DROP TABLE IF EXISTS audit_logs;

DROP TABLE IF EXISTS audit_excluded_tables;

-- 4. Extension và schema.
-- Bỏ các dòng này nếu migration khác (hoặc database có sẵn từ trước) cũng dùng chúng.
DROP EXTENSION IF EXISTS pg_partman;

DROP SCHEMA IF EXISTS partman;
