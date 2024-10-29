fn main() {
    std::process::Command::new("sqlc")
        .arg("generate")
        .spawn()
        .expect("failed to run sqlc generate");
}
