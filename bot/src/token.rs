use anyhow::anyhow;
use base64::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::time::{SystemTime, UNIX_EPOCH};

/// Represents the token's internal payload.
#[derive(Serialize, Deserialize)]
struct TokenPayloadInternal {
    sub: String,
    iat: u64,
    exp: u64,
    aud: String,
}

/// Token represents a bluesky authentication token.
#[derive(Serialize, Deserialize, Debug, PartialOrd, PartialEq, Default)]
struct Token {
    handle: String,
    #[serde(rename(serialize = "accessJwt", deserialize = "accessJwt"))]
    access_jwt: String,
    #[serde(rename(serialize = "refreshJwt", deserialize = "refreshJwt"))]
    refresh_jwt: String,
}

impl Token {
    /// Returns true if the token is expired, false otherwise.
    fn is_expired(&self) -> Result<bool, anyhow::Error> {
        let parts: Vec<&str> = self.access_jwt.split('.').collect();
        let payload_part = parts.get(1).ok_or(anyhow!("Missing payload from token"))?;

        let result = BASE64_STANDARD_NO_PAD.decode(payload_part)?;
        let payload: TokenPayloadInternal = serde_json::from_slice(&result)?;
        let now = SystemTime::now();
        let unix_epoch_seconds = now.duration_since(UNIX_EPOCH)?.as_secs();
        Ok(unix_epoch_seconds - 60 >= payload.exp)
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Token [Handle: {}, AccessJWT: {}, RefreshJWT: {}]",
            self.handle,
            self.access_jwt.get(0..5).unwrap_or(&self.access_jwt),
            self.refresh_jwt.get(0..5).unwrap_or(&self.refresh_jwt),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow;

    #[test]
    fn test_is_expired_true() -> Result<(), anyhow::Error> {
        // Setup
        let payload = TokenPayloadInternal {
            sub: "".to_string(),
            iat: 0,
            exp: 0,
            aud: "".to_string(),
        };
        let json_data = serde_json::to_string(&payload)?;
        let base64_data = BASE64_STANDARD_NO_PAD.encode(json_data);

        let mut token = Token::default();
        token.access_jwt = format!("eyJ0eXAiOiJhdCtqd3QiLCJhbGciOiJFUzI1NksifQ.{}.oWhKfhGWv6omS3oFQ21GX29uzsd5WrfPJyotJMCQ8V44GF1UN2et7sf_JKVB5jkSuJa6kVWERGuKVGgj8AWScA", base64_data);

        // Test
        let result = token.is_expired()?;

        // Assert
        assert_eq!(result, true);
        Ok(())
    }

    #[test]
    fn test_is_expired_false() -> Result<(), anyhow::Error> {
        // Setup
        let now = SystemTime::now();
        let unix_epoch_seconds = now.duration_since(UNIX_EPOCH)?.as_secs() + 100_000;
        let payload = TokenPayloadInternal {
            sub: "".to_string(),
            iat: 0,
            exp: unix_epoch_seconds,
            aud: "".to_string(),
        };
        let json_data = serde_json::to_string(&payload)?;
        let base64_data = BASE64_STANDARD_NO_PAD.encode(json_data);

        let mut token = Token::default();
        token.access_jwt = format!("eyJ0eXAiOiJhdCtqd3QiLCJhbGciOiJFUzI1NksifQ.{}.oWhKfhGWv6omS3oFQ21GX29uzsd5WrfPJyotJMCQ8V44GF1UN2et7sf_JKVB5jkSuJa6kVWERGuKVGgj8AWScA", base64_data);

        // Test
        let result = token.is_expired()?;

        // Assert
        assert_eq!(result, false);
        Ok(())
    }

    #[test]
    fn test_token_deserialize() -> Result<(), anyhow::Error> {
        let data = r#"
        {
            "handle": "cool-bot.bsky.social",
            "email": "cool@gmail.com",
            "emailConfirmed": true,
            "emailAuthFactor": false,
            "accessJwt": "ein.zwei.drei",
            "refreshJwt": "fier.funf.sechs",
            "active": true
        }
"#;

        let token: Token = serde_json::from_str(data)?;

        assert_eq!(
            token,
            Token {
                handle: "cool-bot.bsky.social".to_string(),
                access_jwt: "ein.zwei.drei".to_string(),
                refresh_jwt: "fier.funf.sechs".to_string(),
            }
        );
        Ok(())
    }
}
