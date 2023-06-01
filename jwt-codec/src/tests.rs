use crate::Claims;
use crate::Codec;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
enum Role {
    Guest,
    Administrator,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct User {
    #[serde(rename = "username")]
    name: String,
    role: Role,
}

#[test]
fn test_gen_token() {
    let codec = Codec::hs256(b"TD-Sky's secret key");
    let user = User {
        name: String::from("TD-Sky"),
        role: Role::Administrator,
    };

    let token = codec.gen_token(&user).unwrap();
    assert_eq!(
        &token,
        "eyJhbGciOiJIUzI1NiJ9.\
         eyJ1c2VybmFtZSI6IlRELVNreSIsInJvbGUiOiJBZG1pbmlzdHJhdG9yIn0.\
         ZqlvdCUclvzlpayB6UceubuPoG_EH5UALs7Az-_X5u0"
    );
}

#[test]
fn test_parse_token() {
    let codec = Codec::hs256(b"TD-Sky's secret key");
    let token = "eyJhbGciOiJIUzI1NiJ9.\
         eyJ1c2VybmFtZSI6IlRELVNreSIsInJvbGUiOiJBZG1pbmlzdHJhdG9yIn0.\
         ZqlvdCUclvzlpayB6UceubuPoG_EH5UALs7Az-_X5u0";

    let user: User = codec.parse_token(token).unwrap();
    assert_eq!(
        user,
        User {
            name: String::from("TD-Sky"),
            role: Role::Administrator,
        }
    );
}

#[test]
fn test_registered_claims() {
    let codec = Codec::hs256(b"TD-Sky's secret key");
    let user = User {
        name: String::from("TD-Sky"),
        role: Role::Administrator,
    };
    let claims = Claims::new(user).issuer("TD-Sky".to_owned());

    let token_str = codec.gen_token(&claims).unwrap();
    assert_eq!(
        token_str,
        "eyJhbGciOiJIUzI1NiJ9.\
        eyJpc3MiOiJURC1Ta3kiLCJ1c2VybmFtZSI6IlRELVNreSIsInJvbGUiOiJBZG1pbmlzdHJhdG9yIn0.\
        -678R-ZgSF15ZrryPUbfDLvC3U9P1R3kPZu4Am63l9I"
    )
}
