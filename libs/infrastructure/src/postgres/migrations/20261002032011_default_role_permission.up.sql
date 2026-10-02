-- ==========================================================
-- Dữ liệu mặc định của hệ thống: permission + vai trò ADMIN
-- Chạy được lại nhiều lần (ON CONFLICT DO NOTHING)
-- ==========================================================
-- 1. Danh sách quyền (bổ sung theo module của bạn)
INSERT INTO
    permissions (code, description)
VALUES
    ('USER_VIEW', 'Xem người dùng'),
    ('USER_CREATE', 'Tạo người dùng'),
    ('USER_UPDATE', 'Cập nhật người dùng'),
    ('USER_DELETE', 'Xoá người dùng'),
    ('ROLE_VIEW', 'Xem vai trò'),
    ('ROLE_CREATE', 'Tạo vai trò'),
    ('ROLE_UPDATE', 'Cập nhật vai trò'),
    ('ROLE_DELETE', 'Xoá vai trò'),
    ('PERMISSION_VIEW', 'Xem danh sách quyền'),
    ('SESSION_VIEW', 'Xem phiên đăng nhập'),
    ('SESSION_REVOKE', 'Thu hồi phiên đăng nhập'),
    ('FILE_VIEW', 'Xem file'),
    ('FILE_UPLOAD', 'Tải file lên'),
    ('FILE_DELETE', 'Xoá file'),
    ('AUDIT_LOG_VIEW', 'Xem nhật ký thay đổi') ON CONFLICT (code) DO NOTHING;

-- 2. Vai trò ADMIN: id do database tự sinh (uuidv7), nhận diện bằng TÊN.
--    can_delete/can_update = FALSE để không ai sửa/xoá (kể cả đổi tên) vai trò hệ thống.
--    ON CONFLICT khớp với unique index uq_roles_name (name) WHERE deleted_at IS NULL.
INSERT INTO
    roles (name, description, can_delete, can_update)
VALUES
    (
        'ADMIN',
        'Quản trị hệ thống, có toàn bộ quyền',
        FALSE,
        FALSE
    ) ON CONFLICT (name)
WHERE
    deleted_at IS NULL DO NOTHING;

-- 3. Gán TẤT CẢ permission hiện có cho ADMIN.
--    Migration sau này thêm permission mới thì lặp lại câu lệnh này ở cuối migration đó.
INSERT INTO
    role_permissions (role_id, permission_id)
SELECT
    r.id,
    p.id
FROM
    roles r
    CROSS JOIN permissions p
WHERE
    r.name = 'ADMIN'
    AND r.deleted_at IS NULL ON CONFLICT DO NOTHING;