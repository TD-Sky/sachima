use serde::Deserialize;
use serde::{Serialize, Serializer};

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    #[serde(rename = "username")]
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct UserInfo {
    pub username: String,
    pub roles: [&'static str; 1],
}

#[derive(Debug)]
pub struct Token(pub String);

#[derive(Debug, Serialize)]
struct TokenObject<'token> {
    token: &'token str,
}

impl Serialize for Token {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        TokenObject { token: &self.0 }.serialize(serializer)
    }
}
