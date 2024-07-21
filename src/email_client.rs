use crate::domain::value_objects::SubscriberEmail;

pub struct EmailClient {
    sender: SubscriberEmail,
}

impl EmailClient {
    pub async fn send(
        &self,
        recipient: SubscriberEmail,
        subject: &str,
        html_content: &str,
        text_content: &str,
    ) -> Result<(), String> {
        todo!()
    }
}
