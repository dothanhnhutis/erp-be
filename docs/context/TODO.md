# TODO — trạng thái hiện tại

Cập nhật: 2026-10-23. Cập nhật file này mỗi khi xong / phát sinh việc.

## Vừa xong

Commit `e656aa8` (đã push nhưng build lỗi) + bản sửa ngay sau đó:

- Seed gán lại role ADMIN cho admin (`INSERT INTO user_roles … ON CONFLICT DO NOTHING`).
- `RoleRepo` / `PgRoleRepo`: `find_by_id`, `find_permission_codes_for_role`, `find_existing_permission_codes`, `replace_permissions`.
  - `replace_permissions` trong 1 transaction: `SELECT … WHERE can_update FOR UPDATE` (khoá dòng role + chặn lớp 1 ở tầng DB, không sinh dòng audit `roles`) → kiểm tra mọi code tồn tại (`COUNT(*)`) → chỉ `DELETE` code bị bỏ, `INSERT … ON CONFLICT DO NOTHING` code mới (audit log chỉ ghi thay đổi thật).
  - Đã xoá khối kiểm tra `inserted` (biến không còn tồn tại → build lỗi ở `e656aa8`).
- `RoleService<RR, PC>::update_permissions`: lớp 1 (403) → mọi code tồn tại (400) → lớp 2 (403) → ghi → `bump_version` (lỗi cache chỉ `warn`).
- API `PUT /api/v1/roles/{id}/permissions` — `Require<RoleUpdate>`, body `{ "permission_codes": [...] }` (tối đa 200), trả 204. `role_service` đã có trong `AppState`.
- Bỏ import thừa `HashSet` trong `user_service.rs`.

### Kiểm tra thủ công (chạy trước khi push nếu chưa chạy)

1. `cargo build` sạch lỗi.
2. `sqlx database reset` → `postgres_seed` → `SELECT * FROM user_roles` có dòng admin–ADMIN.
3. Login admin rồi `PUT /api/v1/roles/{id}/permissions`:
   - role ADMIN → 403 "Không thể sửa vai trò hệ thống";
   - role thử (`INSERT INTO roles (name) VALUES ('TEST')`) + `["USER_VIEW"]` → 204, `role_permissions` 1 dòng, Redis `perm:ver` tăng 1;
   - code không tồn tại → **400** liệt kê code sai (không phải 403);
   - role id không tồn tại → 404.

## Đang dở

- [ ] Email mời user: bước gửi trong `UserService::create_user` còn comment; chưa sinh `password_tokens` (type `INIT`); `LapinEmailPublisher` + `declare_email_topology` chưa wiring vào `AppState`/startup.

## Còn mở từ các lần review

### Phân quyền / role

- [ ] 🟡 [role_repo.rs:148](../../libs/infrastructure/src/repositories/role_repo.rs#L148): `FOR UPDATE` xung đột với khoá `FOR KEY SHARE` mà Postgres lấy khi `INSERT` dòng có FK tới `roles` (vd `user_roles` lúc tạo user gán role này) → các thao tác đó phải chờ. Đổi thành `FOR NO KEY UPDATE` (vẫn tuần tự hoá 2 lần sửa cùng role, không chặn insert FK).
- [ ] 🟡 Lớp 2a có TOCTOU: `RoleService::update_permissions` đọc quyền hiện tại của role **ngoài** transaction → nếu người khác vừa thêm quyền mạnh hơn, actor yếu có thể ghi đè và gỡ mất quyền đó. Sửa: `replace_permissions(role_id, expected_current, codes)`, sau khi khoá dòng role thì đọc lại code trong `tx` và so; khác → lỗi riêng → `AppError::Conflict("Vai trò vừa bị thay đổi, hãy thử lại")`.
- [ ] 🟡 `replace_permissions` trả `NotFound` cho cả "role không sửa được" lẫn "code không tồn tại" → service đều báo "Vai trò không tồn tại". Hiếm, nhưng nên tách lỗi.
- [ ] 🟡 `UpdateRolePermissionsRequest`: code rỗng sau trim / quá dài chưa bị chặn rõ ràng.
- [ ] 🟡 Seed không cứu được role ADMIN đã bị xoá mềm: câu lấy `role_id` lọc `deleted_at IS NULL` nên bước "bỏ `deleted_at`" không bao giờ có tác dụng. Tra theo tên, ưu tiên dòng chưa xoá.
- [ ] 🧹 [role_service.rs:56-61](../../libs/application/src/service/role_service.rs#L56-L61): comment "kiểm tra quyền TRƯỚC khi báo code nào không tồn tại" nói ngược với code sau khi đổi thứ tự. Sửa comment; chuyển đoạn lấy `current` xuống ngay trước `ensure_covers` (code sai thì trả lỗi sớm, bớt 1 query).

### HTTP / API

- [ ] 🟡 `Path<Uuid>` sai định dạng trả lỗi mặc định của axum (plain text), không theo format `ApiError` → thêm `PathRejection` vào [error.rs](../../apps/api/src/error.rs) như `JsonRejection`.
- [ ] 🧹 Đổi tên `create_usere_handler` → `create_user_handler` (handler, `mod.rs`, `routes/v1/user.rs`).
- [ ] 🧹 [state.rs](../../apps/api/src/state.rs): `per_cache.clone()` cuối cùng thừa (truyền thẳng `per_cache`); dòng trống thừa sau field `role_service`.
- [ ] 🧹 Thêm request Bruno `bruno_rest/role/update permissions.yml`.

### Email / messaging

- [ ] Email publisher ([email_publisher.rs](../../libs/infrastructure/src/messaging/email_publisher.rs)):
  - kiểm tra `Confirmation`: `Nack` → lỗi; `Ack(Some(_))` (bị trả lại) → lỗi;
  - `BasicPublishOptions { mandatory: true, .. }` để phát hiện message không route được;
  - `tokio::time::timeout` cho publish + confirm;
  - `declare_email_topology` khai báo luôn queue durable + bind (không phụ thuộc worker đã chạy).
- [ ] Tạo user xong mà publish lỗi → user không nhận mail: làm endpoint "gửi lại email mời" hoặc outbox (ghi token + outbox cùng transaction tạo user).
- [ ] `expires_in_hours`: `u64` ở `SetPasswordEmail` nhưng `i64` ở `ResetPasswordEmail` ([messaging.rs](../../libs/shared/src/messaging.rs)) → thống nhất.

### Khác

- [ ] `.env.example` vẫn chứa JWT secret cũ (đã vô hiệu) → đổi thành placeholder.
- [ ] `CreateUserWithRoles` không cần `Deserialize`; cân nhắc đổi tên `NewUser` (giống `NewSession`).
- [ ] Một số file đang CRLF (git cảnh báo, vd `user_service.rs`) → chuyển LF.

## Việc tương lai đã thiết kế

- [ ] Khoá user: trong 1 transaction cập nhật `users.status` + `SessionRepo::revoke_all_for_user(user_id, "FORCED") RETURNING id`; sau commit `SessionCache::mark_revoked_many`. Áp lớp 2 + `ensure_not_self` + "không khoá admin ACTIVE cuối cùng".
- [ ] API sửa role của user, xoá role — theo bảng kiểm tra trong [authorization.md](authorization.md).
- [ ] `audit_logs.changed_by`: set `app.user_id` mỗi transaction.

## Bước tiếp theo đề xuất

1. Chạy kiểm tra thủ công ở trên → commit/push (đưa `main` về trạng thái build được).
2. API khoá user (+ revoke phiên).
3. API sửa role của user, xoá role.
4. Nối email mời (token INIT + publisher đã sửa confirm/mandatory/timeout).

Các mục 🟡/🧹 ở trên làm xen kẽ khi đụng tới file liên quan.

## Tiếp tục ở máy mới

1. Clone repo.
2. Tạo `.env` từ `.env.example`, sinh `JWT_SECRET` mới (vd `openssl rand -base64 32`).
3. `docker compose up -d` → `sqlx database reset` → `cargo run -p infrastructure --bin postgres_seed`.
4. Mở Claude Code tại thư mục repo và nói: *"đọc CLAUDE.md và docs/context/ rồi tiếp tục từ TODO.md"*.

Không commit transcript hội thoại (`~/.claude/projects/*.jsonl`): có chứa secret cũ và đường dẫn máy.
