use sha3::Digest;

pub struct PasswordHash {
    value: String,
}

impl PasswordHash {
    pub fn new_from_password(password: &str) -> Self {
        let hash_bytes = sha3::Sha3_256::digest(password.as_bytes());
        let hash = format!("{:x}", hash_bytes);
        Self { value: hash }
    }
}

impl AsRef<str> for PasswordHash {
    fn as_ref(&self) -> &str {
        &self.value
    }
}
