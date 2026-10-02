-- ==========================================================
-- Down: dữ liệu mặc định (permission + vai trò ADMIN)
-- ==========================================================
-- Gỡ vai trò ADMIN khỏi mọi user (user admin vẫn còn, chỉ mất vai trò)
DELETE FROM user_roles
WHERE
    role_id IN (
        SELECT
            id
        FROM
            roles
        WHERE
            name = 'ADMIN'
            AND deleted_at IS NULL
    );

DELETE FROM role_permissions
WHERE
    role_id IN (
        SELECT
            id
        FROM
            roles
        WHERE
            name = 'ADMIN'
            AND deleted_at IS NULL
    );

DELETE FROM roles
WHERE
    name = 'ADMIN'
    AND deleted_at IS NULL;

-- Sẽ báo lỗi (ON DELETE RESTRICT) nếu vai trò khác đang dùng các quyền này:
-- đó là chủ ý, để không âm thầm tước quyền của vai trò khác.
DELETE FROM permissions
WHERE
    code IN (
        'USER_VIEW',
        'USER_CREATE',
        'USER_UPDATE',
        'USER_DELETE',
        'ROLE_VIEW',
        'ROLE_CREATE',
        'ROLE_UPDATE',
        'ROLE_DELETE',
        'PERMISSION_VIEW',
        'SESSION_VIEW',
        'SESSION_REVOKE',
        'FILE_VIEW',
        'FILE_UPLOAD',
        'FILE_DELETE',
        'AUDIT_LOG_VIEW'
    );