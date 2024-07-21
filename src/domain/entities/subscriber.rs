use crate::domain::value_objects::{SubscriberEmail, SubscriberId, SubscriberName};

pub struct Subscriber {
    pub id: SubscriberId,
    pub email: SubscriberEmail,
    pub name: SubscriberName,
}
