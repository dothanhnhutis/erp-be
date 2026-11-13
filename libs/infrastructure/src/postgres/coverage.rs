//! Kiểm tra mọi bảng đã được gắn trigger bắt buộc chưa — phát hiện migration quên
//! `SELECT sync_updated_at_triggers(); SELECT sync_audit_triggers();` ở cuối file.
//! Điều kiện phải khớp với 2 hàm SQL cùng tên trong migrations.

use sqlx::PgPool;

#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
pub struct MissingTrigger {
    pub table_name: String,
    /// `audit` | `updated_at`
    pub kind: String,
}

pub async fn missing_triggers(pool: &PgPool) -> Result<Vec<MissingTrigger>, sqlx::Error> {
    sqlx::query_as(
        r#"
        WITH t AS (
            SELECT c.oid, c.relname::text AS relname
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind IN ('r', 'p')
              AND NOT c.relispartition
        )
        SELECT t.relname AS table_name, 'audit' AS kind
        FROM t
        WHERE NOT EXISTS (SELECT 1 FROM audit_excluded_tables e WHERE e.table_name = t.relname)
          AND NOT EXISTS (
              SELECT 1 FROM pg_trigger g
              WHERE g.tgrelid = t.oid
                AND g.tgname IN (left('trg_audit_ins_del_' || t.relname, 63),
                                 left('trg_audit_upd_' || t.relname, 63))
              HAVING count(*) = 2
          )
        UNION ALL
        SELECT t.relname, 'updated_at'
        FROM t
        WHERE EXISTS (
                SELECT 1 FROM pg_attribute a
                WHERE a.attrelid = t.oid AND a.attname = 'updated_at' AND NOT a.attisdropped
              )
          AND NOT EXISTS (
                SELECT 1 FROM pg_trigger g
                WHERE g.tgrelid = t.oid AND g.tgname = left('trg_updated_at_' || t.relname, 63)
              )
        ORDER BY 1, 2
        "#,
    )
    .fetch_all(pool)
    .await
}
