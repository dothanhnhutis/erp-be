# CLAUDE.md

## Cách làm việc với user
- **Mặc định chỉ tư vấn**: KHÔNG sửa code/file khi user chưa cho phép rõ ràng. Gợi ý, review, code mẫu → trả lời bằng text trong hội thoại. User tự sửa, tự commit, tự push.
- Trả lời bằng **tiếng Việt**.
- **Không nhắc** chuyện sửa thẳng file migration cũ: user luôn revert hoặc `sqlx database reset` rồi migrate lại.
- Khi review: đọc `git status` / `git diff HEAD` + diagnostics, liệt kê theo mức độ (🔴 / 🟠 / 🟡 / 🧹), có link file:dòng.

## Kiến trúc
Cargo workspace, kiến trúc hexagonal:

| Crate | Vai trò |
|---|---|
| `apps/api` | HTTP (axum): handlers, extractors, middleware, routes, `AppState` |
| `libs/domain` | Entity + port trait (`UserRepo`, `SessionRepo`, `RoleRepo`, `SessionCache`, `PermissionCache`) |
| `libs/application` | Service (`AuthService`, `UserService`, `RoleService`), DTO, `AppError`, security (tokens, password, authorization), port `EmailPublisher` |
| `libs/infrastructure` | Adapter: Postgres (sqlx), Redis, RabbitMQ (lapin + deadpool), migrations, bin `postgres_seed` |
| `libs/shared` | Config (`AppConfig`, `TokenConfig`), hợp đồng message (`EmailJob`) |

## Lệnh hay dùng
```bash
docker compose up -d                                   # postgres (5433), redis, rabbitmq
sqlx database reset                                    # drop + migrate lại
cargo run -p infrastructure --bin postgres_seed        # tạo/khôi phục admin (cần ADMIN_EMAIL, ADMIN_PASSWORD)
cargo build
```
`.env` không nằm trong git — tạo từ `.env.example` và sinh `JWT_SECRET` mới.

## Ngữ cảnh chi tiết
- [docs/context/TODO.md](docs/context/TODO.md) — **đọc trước**: trạng thái hiện tại, việc đang dở, bước tiếp theo.
- [docs/context/auth-session.md](docs/context/auth-session.md) — thiết kế login / refresh / logout / cache phiên đã chốt.
- [docs/context/authorization.md](docs/context/authorization.md) — phân quyền 3 lớp.
