# TODO — trạng thái hiện tại

Cập nhật: 2026-10-22. Cập nhật file này mỗi khi xong / phát sinh việc.

## Đang dở (chưa commit tại thời điểm viết)
Tính năng: admin tạo user + gán role + (sau này) gửi email mời qua RabbitMQ, và phân quyền 3 lớp ([authorization.md](authorization.md)).

- [ ] 🔴 **Build đang lỗi**: `PgRoleRepo` chưa cài `find_by_id`, `find_permission_codes_for_role` ([role_repo.rs](../../libs/infrastructure/src/repositories/role_repo.rs)). Cần map entity `Role` (`id, name, description, status, can_update, can_delete, deleted_at`).
- [ ] 🔴 **Seed mất bước gán role**: trong [postgres_seed.rs](../../libs/infrastructure/src/bin/postgres_seed.rs), câu `INSERT INTO user_roles (user_id, role_id) … ON CONFLICT DO NOTHING` đã bị thay bằng `UPDATE roles` → seed không còn gán role ADMIN cho user admin. Thêm lại câu đó (giữ cả các câu khôi phục mới).
- [ ] `RoleService::update_permissions` ([role_service.rs](../../libs/application/src/service/role_service.rs)) mới là khung: còn thiếu kiểm tra mọi code tồn tại, thay `role_permissions` trong 1 transaction, `perm_cache.bump_version()`; chưa đưa vào `AppState` và route.
- [ ] Email mời user: bước gửi trong `UserService::create_user` còn comment; chưa sinh `password_tokens` (type `INIT`); `LapinEmailPublisher` + `declare_email_topology` chưa wiring vào `AppState`/startup.

## Còn mở từ các lần review
- [ ] Email publisher ([email_publisher.rs](../../libs/infrastructure/src/messaging/email_publisher.rs)):
  - kiểm tra `Confirmation`: `Nack` → lỗi; `Ack(Some(_))` (bị trả lại) → lỗi;
  - `BasicPublishOptions { mandatory: true, .. }` để phát hiện message không route được;
  - `tokio::time::timeout` cho publish + confirm;
  - `declare_email_topology` khai báo luôn queue durable + bind (không phụ thuộc worker đã chạy).
- [ ] Tạo user xong mà publish lỗi → user không nhận mail: làm endpoint "gửi lại email mời" hoặc outbox (ghi token + outbox cùng transaction tạo user).
- [ ] `.env.example` vẫn chứa JWT secret cũ (đã vô hiệu) → đổi thành placeholder.
- [ ] Đổi tên `create_usere_handler` → `create_user_handler` (handler, `mod.rs`, `routes/v1/user.rs`).
- [ ] Import thừa `std::collections::HashSet` trong [user_service.rs](../../libs/application/src/service/user_service.rs).
- [ ] `expires_in_hours`: `u64` ở `SetPasswordEmail` nhưng `i64` ở `ResetPasswordEmail` ([messaging.rs](../../libs/shared/src/messaging.rs)) → thống nhất.
- [ ] `CreateUserWithRoles` không cần `Deserialize`; cân nhắc đổi tên `NewUser` (giống `NewSession`).
- [ ] Một số file đang CRLF (git cảnh báo) → chuyển LF.

## Việc tương lai đã thiết kế
- [ ] Khoá user: trong 1 transaction cập nhật `users.status` + `SessionRepo::revoke_all_for_user(user_id, "FORCED") RETURNING id`; sau commit `SessionCache::mark_revoked_many`. Áp lớp 2 + `ensure_not_self` + "không khoá admin ACTIVE cuối cùng".
- [ ] API sửa role của user, xoá role — theo bảng kiểm tra trong [authorization.md](authorization.md).
- [ ] `audit_logs.changed_by`: set `app.user_id` mỗi transaction.

## Bước tiếp theo đề xuất
1. Sửa 2 mục 🔴 → `cargo build` sạch.
2. Hoàn thiện `RoleService` + API sửa permission role.
3. API khoá user (+ revoke phiên).
4. Nối email mời (token INIT + publisher đã sửa confirm/mandatory/timeout).

## Tiếp tục ở máy mới
1. Clone repo.
2. Tạo `.env` từ `.env.example`, sinh `JWT_SECRET` mới (vd `openssl rand -base64 32`).
3. `docker compose up -d` → `sqlx database reset` → `cargo run -p infrastructure --bin postgres_seed`.
4. Mở Claude Code tại thư mục repo và nói: *"đọc CLAUDE.md và docs/context/ rồi tiếp tục từ TODO.md"*.

Không commit transcript hội thoại (`~/.claude/projects/*.jsonl`): có chứa secret cũ và đường dẫn máy.
