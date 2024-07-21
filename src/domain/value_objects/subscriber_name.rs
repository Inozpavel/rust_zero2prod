#[derive(Eq, PartialEq, Debug)]
pub struct SubscriberName(String);

impl SubscriberName {
    pub fn parse(s: String) -> Result<Self, String> {
        let is_empty_or_whitespace = s.trim().is_empty();

        let is_too_long = s.len() > 256;

        static FORBIDDEN_CHARS: [char; 8] = ['/', '\\', '(', ')', '<', '>', '{', '}'];

        let contains_forbidden_char = s.chars().any(|c| FORBIDDEN_CHARS.contains(&c));

        if is_empty_or_whitespace || is_too_long || contains_forbidden_char {
            return Err(format!("{} is not valid subscriber name", s));
        }
        Ok(Self(s))
    }
}

impl AsRef<str> for SubscriberName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
