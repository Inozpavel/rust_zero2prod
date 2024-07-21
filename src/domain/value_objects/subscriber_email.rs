pub struct SubscriberEmail(String);

impl SubscriberEmail {
    pub fn parse(s: String) -> Result<Self, String> {
        if validator::ValidateEmail::validate_email(&s) {
            Ok(Self(s))
        } else {
            Err(format!("{} is not valid email", s))
        }
    }
}
impl AsRef<str> for SubscriberEmail {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fake::Fake;

    #[test]
    fn empty_string_is_rejected() {
        let email = "".to_string();
        assert!(SubscriberEmail::parse(email).is_err());
    }
    #[test]
    fn email_missing_at_symbol_is_rejected() {
        let email = "ursuladomain.com".to_string();
        assert!(SubscriberEmail::parse(email).is_err());
    }
    #[test]
    fn email_missing_subject_is_rejected() {
        let email = "@domain.com".to_string();
        assert!(SubscriberEmail::parse(email).is_err());
    }

    #[test]
    fn valid_emails_are_parsed_successfully() {
        let email = fake::faker::internet::en::SafeEmail().fake::<String>();
        println!("email: {email}");
        assert!(SubscriberEmail::parse(email).is_ok())
    }
}
