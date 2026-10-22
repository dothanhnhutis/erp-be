# Phân quyền — 3 lớp chống leo thang quyền

Cập nhật: 2026-10-22. Trạng thái việc dở xem [TODO.md](TODO.md).

## Vấn đề cần chặn
- Người có `USER_CREATE` tạo user mới với role `ADMIN` → leo thang quyền.
- Người có `ROLE_UPDATE` gỡ permission khỏi role `ADMIN` → admin mất quyền và **không tự khôi phục được** (vì quy tắc "chỉ cấp quyền mình có").
- Người có `USER_UPDATE` gỡ role ADMIN khỏi admin / khoá tài khoản admin.

## Nền: `Actor` ([authorization.rs](../../libs/application/src/security/authorization.rs))
- `Actor { user_id, permissions: HashSet<String> }`, lấy từ request bằng extractor `CurrentActor` ([actor.rs](../../apps/api/src/extractors/actor.rs)) — ghép `Me` + `Permissions` do middleware chèn.
- `ensure_covers(codes, action)`: actor phải có **mọi** permission trong `codes`, nếu không → 403 kèm danh sách quyền thiếu.
- `ensure_not_self(target, action)`: chặn tự khoá / tự gỡ role của chính mình.

## Lớp 1 — role hệ thống bất biến
- Bảng `roles` có `can_update`, `can_delete`; role `ADMIN` đặt cả hai = `FALSE` (migration `default_role_permission`).
- `RoleService` (sửa permission, đổi tên, xoá role) **kiểm tra đầu tiên** `can_update` / `can_delete` → 403 "Không thể sửa vai trò hệ thống". Không ngoại lệ, kể cả admin.
- Chặn thêm ở tầng DB: `PgRoleRepo::replace_permissions` mở transaction bằng `UPDATE roles … WHERE can_update RETURNING id` (vừa khoá dòng role, vừa từ chối role hệ thống dù service có sót kiểm tra).
- Thứ tự kiểm tra khi sửa permission role: lớp 1 (403) → mọi code tồn tại (400) → lớp 2 (403). "Code tồn tại" phải đứng trước lớp 2: actor không bao giờ có code không tồn tại, nên để sau thì lớp 2 trả nhầm 403.
- Permission của ADMIN chỉ đổi qua migration (bước cuối migration luôn cấp mọi permission mới cho ADMIN).
- Tuỳ chọn sau này: trigger DB chặn ghi `role_permissions` của role `can_update = false`, chừa bypass cho migration/seed qua `current_setting('app.bypass_system_role', true)`.

## Lớp 2 — chỉ thao tác lên thứ "không mạnh hơn mình"
> Actor chỉ được thao tác lên role R khi permission(R) ⊆ permission(actor),
> và lên user U khi permission(U) ⊆ permission(actor).

Chọn quy tắc theo **permission** (không theo "phải giữ role đó") vì: giữ role ⇒ có đủ permission của role nên đã bao gồm; và chặn đúng điều cần chặn là cấp quyền mình không có.

| API | Kiểm tra | Trạng thái |
|---|---|---|
| Tạo user | role tồn tại + ACTIVE; `ensure_covers(permission của các role được gán)` | ✅ đã làm (`UserService::create_user`) |
| Sửa role của user | `ensure_not_self` + `ensure_covers(quyền hiện tại của target)` + `ensure_covers(quyền của role mới)` | chưa có API |
| Khoá / xoá user | `ensure_not_self` + `ensure_covers(quyền hiện tại của target)` → revoke mọi phiên | chưa có API |
| Sửa permission của role | `can_update` + mọi code tồn tại + `ensure_covers(quyền hiện tại của role)` + `ensure_covers(quyền mới)` + `bump_version` | ✅ `PUT /api/v1/roles/{id}/permissions` (`RoleService::update_permissions`) — còn TOCTOU ở lớp 2a, xem TODO |
| Xoá role | `can_delete` + `ensure_covers(quyền hiện tại của role)` | chưa có |

Thêm: **không gỡ role ADMIN khỏi admin ACTIVE cuối cùng** (khi sửa role user / khoá user).

Quyền hiện tại của user đích: `RoleRepo::find_permission_codes_for_user`. Của role: `find_permission_codes_for_role` / `find_active_role_permissions`.

Lưu ý: permission của actor lấy từ cache permission (`perm:v{ver}:user:{id}`); khi sửa role/permission phải `bump_version` (đổi permission của role) hoặc `invalidate_user` (gán/gỡ role của 1 user).

## Lớp 3 — lối thoát khẩn cấp ([postgres_seed.rs](../../libs/infrastructure/src/bin/postgres_seed.rs))
Chạy được chỉ khi có quyền truy cập server/DB. Trong 1 transaction:
1. Đảm bảo user admin tồn tại và **có role ADMIN** (`INSERT INTO user_roles … ON CONFLICT DO NOTHING`).
2. Role ADMIN: `status = 'ACTIVE'`, bỏ `deactivated_at`/`deleted_at`, `can_update = can_delete = FALSE`.
3. Cấp lại **mọi** permission cho ADMIN (`INSERT INTO role_permissions SELECT … FROM permissions ON CONFLICT DO NOTHING`).
4. Mở khoá tài khoản admin nếu đã có mật khẩu.

Sau commit: `INCR perm:ver` trong Redis để cache permission cũ bị bỏ ngay.

Hạn chế hiện tại: câu lấy `role_id` lọc `deleted_at IS NULL` → nếu ADMIN bị xoá mềm trực tiếp trong DB thì seed báo lỗi thay vì khôi phục (xem TODO).

## Truy vết
`audit_logs` có trigger trên `role_permissions`, `user_roles`… → tra được ai gỡ quyền gì. Vì vậy khi thay danh sách (vd `replace_permissions`) chỉ `DELETE` phần bị bỏ và `INSERT … ON CONFLICT DO NOTHING` phần mới — không xoá hết rồi chèn lại, để log không bị nhiễu. Cột `changed_by` chỉ đúng khi app set biến session (vd `set_config('app.user_id', …, true)`) trong mỗi transaction — chưa làm.
