use uuid::Uuid;

pub struct SubscriberId(Uuid);

impl SubscriberId {
    pub fn new() -> Self {
        Self::default()
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
