# TODO — trạng thái hiện tại

Cập nhật: 2026-10-22. Cập nhật file này mỗi khi xong / phát sinh việc.

## ⚠️ Sửa trước khi commit/push (xoá mục này khi xong)
- [ ] 🔴 **Build lỗi**: [role_repo.rs:193](../../libs/infrastructure/src/repositories/role_repo.rs#L193) còn khối `if inserted as usize != codes.len()` nhưng biến `inserted` đã bỏ → xoá cả khối (đã có kiểm tra `COUNT(*)` ở trên).
- [ ] 🟠 **Thứ tự kiểm tra sai**: [role_service.rs:56-80](../../libs/application/src/service/role_service.rs#L56-L80) — khối `if existing.len() != new_codes.len()` vẫn nằm **sau** 2 dòng `ensure_covers` → gửi code không tồn tại vẫn ra 403 thay vì 400 (actor không bao giờ có code không tồn tại). Chuyển khối `if` lên trước `ensure_covers`, sửa comment dòng 56. Thứ tự đúng: lớp 1 → code tồn tại (400) → lớp 2 (403).

## Vừa xong (cùng commit này)
- Seed gán lại role ADMIN cho admin (`INSERT INTO user_roles … ON CONFLICT DO NOTHING`).
- `RoleRepo` / `PgRoleRepo`: `find_by_id`, `find_permission_codes_for_role`, `find_existing_permission_codes`, `replace_permissions`.
  - `replace_permissions` trong 1 transaction: `UPDATE roles … WHERE can_update RETURNING id` (khoá dòng role + chặn lớp 1 ở tầng DB) → kiểm tra mọi code tồn tại → chỉ `DELETE` code bị bỏ, `INSERT … ON CONFLICT DO NOTHING` code mới (audit log chỉ ghi thay đổi thật).
- `RoleService<RR, PC>::update_permissions` + `bump_version` (lỗi cache chỉ `warn`).
- API `PUT /api/v1/roles/{id}/permissions` — `Require<RoleUpdate>`, body `{ "permission_codes": [...] }` (tối đa 200), trả 204. Đã đưa `role_service` vào `AppState`.

### Kiểm tra thủ công sau khi sửa 2 mục trên
1. `cargo build` sạch lỗi.
2. `sqlx database reset` → `postgres_seed` → `SELECT * FROM user_roles` có dòng admin–ADMIN.
3. Login admin rồi `PUT /api/v1/roles/{id}/permissions`:
   - role ADMIN → 403 "Không thể sửa vai trò hệ thống";
   - role thử (`INSERT INTO roles (name) VALUES ('TEST')`) + `["USER_VIEW"]` → 204, `role_permissions` 1 dòng, Redis `perm:ver` tăng 1;
   - code không tồn tại → 400 liệt kê code sai;
   - role id không tồn tại → 404.

## Đang dở
- [ ] Email mời user: bước gửi trong `UserService::create_user` còn comment; chưa sinh `password_tokens` (type `INIT`); `LapinEmailPublisher` + `declare_email_topology` chưa wiring vào `AppState`/startup.

## Còn mở từ các lần review
- [ ] 🟡 Lớp 2a có TOCTOU: `RoleService::update_permissions` đọc quyền hiện tại của role **ngoài** transaction → nếu người khác vừa thêm quyền mạnh hơn, actor yếu có thể ghi đè và gỡ mất quyền đó. Sửa: `replace_permissions(role_id, expected_current, codes)`, sau khi khoá dòng role thì đọc lại code trong `tx` và so; khác → lỗi riêng → `AppError::Conflict("Vai trò vừa bị thay đổi, hãy thử lại")`.
- [ ] 🟡 `replace_permissions` trả `NotFound` cho cả "role không sửa được" lẫn "code không tồn tại" (cache actor cũ) → service đều báo "Vai trò không tồn tại". Hiếm, nhưng nên tách lỗi.
- [ ] 🟡 `Path<Uuid>` sai định dạng trả lỗi mặc định của axum (plain text), không theo format `ApiError` → thêm `PathRejection` vào [error.rs](../../apps/api/src/error.rs) như `JsonRejection`.
- [ ] 🟡 `UpdateRolePermissionsRequest`: code rỗng sau trim / quá dài chưa bị chặn rõ ràng.
- [ ] 🟡 Seed không cứu được role ADMIN đã bị xoá mềm: câu lấy `role_id` lọc `deleted_at IS NULL` nên bước "bỏ `deleted_at`" không bao giờ có tác dụng. Tra theo tên, ưu tiên dòng chưa xoá.
- [ ] 🧹 [state.rs](../../apps/api/src/state.rs): `per_cache.clone()` cuối cùng thừa (truyền thẳng `per_cache`); dòng trống thừa sau field `role_service`.
- [ ] 🧹 Thêm request Bruno `bruno_rest/role/update permissions.yml`.
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
1. Sửa 2 mục ở "⚠️ Sửa trước khi commit/push" → chạy kiểm tra thủ công → commit/push.
2. API khoá user (+ revoke phiên).
3. API sửa role của user, xoá role.
4. Nối email mời (token INIT + publisher đã sửa confirm/mandatory/timeout).

## Tiếp tục ở máy mới
1. Clone repo.
2. Tạo `.env` từ `.env.example`, sinh `JWT_SECRET` mới (vd `openssl rand -base64 32`).
3. `docker compose up -d` → `sqlx database reset` → `cargo run -p infrastructure --bin postgres_seed`.
4. Mở Claude Code tại thư mục repo và nói: *"đọc CLAUDE.md và docs/context/ rồi tiếp tục từ TODO.md"*.

Không commit transcript hội thoại (`~/.claude/projects/*.jsonl`): có chứa secret cũ và đường dẫn máy.
