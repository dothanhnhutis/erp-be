// `sqlx::migrate!` nhúng file SQL vào binary lúc compile.
// Báo cargo build lại crate khi thư mục migrations thay đổi (thêm/sửa file),
// nếu không binary có thể vẫn mang bộ migration cũ.
fn main() {
    println!("cargo:rerun-if-changed=migrations");
}
