# Auth & session — quyết định thiết kế đã chốt

Cập nhật: 2026-10-22. File này ghi **vì sao** làm như vậy; trạng thái việc dở xem [TODO.md](TODO.md).

## Token

| | Access token | Refresh token |
|---|---|---|
| Dạng | JWT HS256, claims `sub` (user id), `sid` (session id), `iat`, `exp` | `<session_id>.<32 byte ngẫu nhiên base64url>` (43 ký tự) |
| Lưu ở server | không | DB chỉ lưu `hex(sha256(secret))` (`CHAR(64)`) |
| Thời hạn | 15 phút (`ACCESS_TOKEN_TTL_SECS=900`), leeway verify 30s | 30 ngày trượt (`REFRESH_TOKEN_TTL_SECS`), tối đa 6 tháng (`MAX_REFRESH_TOKEN_TTL_SECS` → `absolute_expires_at`) |

Code: [tokens.rs](../../libs/application/src/security/tokens.rs).

- Tiền tố `session_id` cho phép tra session bằng primary key; so hash bằng `hashes_equal` (constant-time).
- Web: token trong cookie httpOnly. Access cookie `path=/`, `SameSite=Lax`; refresh cookie `path=/api/v1/auth`, `SameSite=Strict`. Body chỉ trả thời điểm hết hạn.
- Mobile/desktop: token trả trong body; refresh gửi qua header `x-refresh-token`.
- Helper dùng chung: `auth_response`, `clear_auth_cookies` trong [helpers/cookie.rs](../../apps/api/src/helpers/cookie.rs). Khi xoá cookie phải set đúng `domain` như lúc tạo.

## Refresh ([auth_service.rs](../../libs/application/src/service/auth_service.rs) `refresh`)
1. `find_by_id(session_id)` → kiểm tra revoked, `expires_at`, `absolute_expires_at`.
2. So hash secret:
   - khớp `refresh_token_hash` → hợp lệ;
   - khớp `previous_token_hash` và `now - rotated_at ≤ 30s` → **409** (race nhiều tab), không revoke;
   - khớp `previous_token_hash` ngoài grace → **dùng lại token cũ = nghi bị đánh cắp** → revoke `REUSED` + tombstone → 401;
   - không khớp gì → 401 (không revoke, tránh bị phá phiên khi chỉ biết session id).
3. User phải `Active`.
4. UA/IP khác lúc login → **chỉ log warn**, không chặn. Lý do: UA đổi mỗi lần trình duyệt/app cập nhật, IP đổi khi đổi mạng → false positive cao; kẻ trộm token cũng dễ giả UA. Reuse detection đáng tin hơn.
5. `ROTATE_SESSION`: `UPDATE … WHERE refresh_token_hash = $old AND revoked_at IS NULL AND absolute_expires_at > NOW()` (compare-and-swap). Không có dòng → 409.
6. `expires_at = min(now + 30 ngày, absolute_expires_at)`.

Handler refresh trả `Response` (không phải `Result<impl IntoResponse, ApiError>`) và nhận `Result<RefreshToken, ApiError>`: để nhánh 401 (kể cả cookie hỏng ở extractor) trả được `(clear_auth_cookies(jar), err)` — dùng `?` sẽ làm mất `jar`. 409 thì không xoá cookie.

**Refresh không huỷ access token cũ** (vẫn sống ≤15 phút). Đã cân nhắc chặn bằng `iat < rotated_at` nhưng bỏ: không tăng bảo mật đáng kể, làm request song song bị 401. Các ca nguy hiểm (logout, `REUSED`, khoá user) đều chặn ngay nhờ tombstone.

## `authentication` (middleware `require_auth`)
- Đọc cache Redis `session:{user_id}:{sid}` → `Hit` / `Revoked` (tombstone) / `Miss` (đọc DB).
- Chặn: `revoked_at`, `expires_at <= now` (→ revoke `EXPIRED`), user khác `Active` (→ revoke `FORCED`). Lỗi revoke chỉ log, vẫn trả 401.
- **Không gia hạn `expires_at` ở đây** — chỉ refresh gia hạn. Lý do: client hoạt động bắt buộc refresh mỗi 15 phút, sliding theo từng request chỉ hơn tối đa 15 phút mà tốn ghi DB/Redis.
- Throttle ghi `last_seen_at` mỗi `SESSION_DB_SYNC_SECS` (60s). `touch_last_seen` trả `bool` (`WHERE revoked_at IS NULL`): `false` → session đã bị revoke trong DB nhưng cache còn bản cũ → tombstone + 401.
- Chỉ `put` cache khi cache miss hoặc vừa touch.

## Cache phiên + tombstone ([cache/session.rs](../../libs/infrastructure/src/cache/session.rs))
- Revoke **không xoá key** mà ghi `__revoked__` (TTL = `access_ttl + 60s`, đủ cho JWT cũ hết hạn + leeway).
- `put` dùng Lua `SAFE_PUT`: không ghi đè tombstone; trả `false` nếu bị chặn → caller trả 401.
- Thứ tự luôn: **DB trước, tombstone sau** (Redis lỗi thì DB vẫn đúng; ngược lại có thể làm phiên "sống lại").
- Hai lớp chống race logout ↔ request song song: tombstone (khoảng hở ≈ 0) + touch trả `bool` (dự phòng khi Redis lỗi, khoảng hở ≤ 60s).

## Revoke reason (`user_sessions.revoke_reason`)
`LOGOUT` (tự logout) · `FORCED` (logout mọi thiết bị / tài khoản bị khoá) · `USER` (user thu hồi) · `EXPIRED` · `REUSED` (refresh token cũ bị dùng lại).

## Login
- Luôn chạy `verify_password` (dùng `DUMMY_HASH` khi không có user) → chống dò email qua thời gian phản hồi.
- Chỉ `Active` được login; `PendingPassword` trả cùng thông báo với sai mật khẩu; `Deactivated` → "Tài khoản đã bị vô hiệu hóa".
- Email so khớp không phân biệt hoa thường (index `lower(email)`); password `max = 255` (chống DoS argon2).
