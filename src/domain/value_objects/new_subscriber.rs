use crate::domain::value_objects::subscriber_email::SubscriberEmail;
use crate::domain::value_objects::subscriber_name::SubscriberName;

pub struct NewSubscriber {
    pub email: SubscriberEmail,
    pub name: SubscriberName,
}
