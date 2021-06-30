#[derive(Debug)]
pub struct Error(String);
impl Error {
    pub(crate) fn new(msg: impl Into<String>) -> Self {
        Self(msg.into())
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "sxd_html error : {}", self.0)
    }
}

impl std::error::Error for Error {}
