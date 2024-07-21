use strum_macros::AsRefStr;

#[derive(Debug, AsRefStr, Eq, PartialEq)]
pub enum ConfirmationStatus {
    PendingConfirmation,
    Confirmed,
}
