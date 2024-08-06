use crate::error::DomainError;
use uuid::Uuid;

#[derive(Debug, Eq, PartialEq)]
pub struct SubscriberId(Uuid);

impl SubscriberId {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn parse(s: &str) -> Result<Self, DomainError> {
        let id = Uuid::parse_str(s).map_err(|_| format!("Incorrect subscriber id {}", s))?;
        Ok(Self(id))
    }
}

impl Default for SubscriberId {
    fn default() -> Self {
        Self(Uuid::now_v7())
    }
}

impl AsRef<Uuid> for SubscriberId {
    fn as_ref(&self) -> &Uuid {
        &self.0
    }
}
