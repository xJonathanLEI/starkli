fn main() {
    vergen::EmitBuilder::builder()
        .git_sha(true)
        .emit()
        .expect("Failed to acquire build-time information");
}
