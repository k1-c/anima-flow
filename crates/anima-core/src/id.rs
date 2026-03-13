pub fn generate() -> String {
    ulid::Ulid::new().to_string()
}
