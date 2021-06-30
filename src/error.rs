#[derive(Debug)]
pub struct Error(String);
impl Error {
    pub(crate) fn new(msg: impl Into<String>) -> Self {
        Self(msg.into())
    }
}
