use crate::domain::value_objects::SubscriberEmail;
use serde::Serialize;

#[derive(Debug)]
pub struct EmailClient {
    http_client: reqwest::Client,
    url: String,
    authorization_token: String,
    sender_email: SubscriberEmail,
}

impl EmailClient {
    pub fn new(
        base_address: String,
        authorization_token: String,
        sender_email: SubscriberEmail,
    ) -> Self {
        let url = format!("{}/email", base_address);
        Self {
            http_client: reqwest::Client::new(),
            url,
            authorization_token,
            sender_email,
        }
    }
    pub async fn send(
        &self,
        recipient: &SubscriberEmail,
        subject: &str,
        html_content: &str,
        text_content: &str,
    ) -> Result<(), anyhow::Error> {
        let request_body = RequestBody {
            to: recipient.as_ref().to_owned(),
            from: self.sender_email.as_ref().to_owned(),
            text_content: text_content.to_string(),
            html_content: html_content.to_string(),
            subject: subject.to_string(),
        };

        println!("Email sent: {:?}", request_body);
        // self.http_client
        //     .post(&self.url)
        //     .header("X-Postmark-Server-Token", &self.authorization_token)
        //     .json(&request_body)
        //     .send()
        //     .await?;
        Ok(())
    }
}

#[derive(Serialize, Debug)]
struct RequestBody {
    from: String,
    to: String,
    subject: String,
    html_content: String,
    text_content: String,
}
