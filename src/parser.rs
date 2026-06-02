use hmac::{Hmac, Mac};
use sha1::Sha1;
use std::fmt::Display;
use thiserror::Error;

#[derive(Clone, Default)]
pub struct Parser {
    check_key: bool,
    hasher: Signature,
}

pub type ChannelParseResult<T> = Result<T, ParseError>;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Input string is empty")]
    EmptyString,
    #[error("Channel segment has invalid format: expected 'data.hash', got '{0}'")]
    MalformedChannel(String),
    #[error("Signature mismatch for data: '{0}'")]
    InvalidSignature(String),
}

impl Parser {
    pub fn new(check_key: bool, hasher: Signature) -> Self {
        Self { check_key, hasher }
    }

    pub fn signature_check_on(&mut self) {
        self.check_key = true;
    }

    pub fn signature_check_off(&mut self) {
        self.check_key = false;
    }

    pub fn set_signature(&mut self, signature: Signature) {
        self.hasher = signature;
    }

    pub fn parse(&self, line: &str) -> ChannelParseResult<Vec<Channel>> {
        if line.is_empty() {
            return Err(ParseError::EmptyString);
        }

        let mut channels = Vec::new();

        for segment in line.split('/') {
            let (data, provided_hash) = if let Some((data, hash)) = segment.split_once('.') {
                (data, Some(hash))
            } else {
                // No dot — only allowed if signature check is OFF
                if self.check_key {
                    return Err(ParseError::MalformedChannel(segment.to_string()));
                }
                (segment, None)
            };

            // Validate signature if required
            if self.check_key {
                let expected_hash = self.hasher.get_digest(data);
                if Some(expected_hash.as_str()) != provided_hash {
                    return Err(ParseError::InvalidSignature(data.to_string()));
                }
            }

            // Parse data into private/public channels (format: "priv:pub")
            let mut parts = data.split(':');
            if let Some(priv_part) = parts.next() {
                channels.push(Channel::create_private(priv_part.to_string()));
            }
            if let Some(pub_part) = parts.next() {
                channels.push(Channel::create_public(pub_part.to_string()));
            }
            // Ignore extra parts beyond second (e.g., "a:b:c" → only a, b)
        }

        Ok(channels)
    }

    pub fn get_key(&self) -> &str {
        self.hasher.key()
    }

    pub fn get_status(&self) -> String {
        if self.check_key {
            format!("enabled with key {}", self.hasher.key())
        } else {
            "disabled".to_string()
        }
    }
}

type HmacSha1 = Hmac<Sha1>;

#[derive(Debug, Clone)]
pub struct Signature {
    key: String,
    mac: HmacSha1,
}

impl Default for Signature {
    fn default() -> Self {
        Self {
            key: String::new(),
            mac: HmacSha1::new_from_slice(b"").expect("empty key is valid"),
        }
    }
}

impl Signature {
    pub fn new(key: String) -> Self {
        let mac = HmacSha1::new_from_slice(key.as_bytes()).expect("HMAC key must be valid");
        Self { key, mac }
    }

    pub fn get_digest(&self, data: &str) -> String {
        let mut mac = self.mac.clone();
        mac.update(data.as_bytes());
        mac.finalize()
            .into_bytes()
            .iter()
            .map(|&byte| format!("{:02x}", byte))
            .collect()
    }

    pub fn key(&self) -> &str {
        &self.key
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum ChannelType {
    Private,
    Public,
    Unknown,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Channel {
    kind: ChannelType,
    number: String,
}

impl Channel {
    pub fn create_private(number: String) -> Self {
        Self {
            kind: ChannelType::Private,
            number,
        }
    }

    pub fn create_public(number: String) -> Self {
        Self {
            kind: ChannelType::Public,
            number,
        }
    }

    pub fn create_unknown(number: String) -> Self {
        Self {
            kind: ChannelType::Unknown,
            number,
        }
    }

    pub fn get_kind(&self) -> ChannelType {
        self.kind.clone()
    }
}

impl Display for Channel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.number)
    }
}

impl TryFrom<Vec<u8>> for Channel {
    type Error = &'static str;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Err("Channel bytes is empty!");
        }

        let hex_string: String = value
            .into_iter()
            .map(|byte| format!("{:02x}", byte))
            .collect();

        Ok(Channel::create_unknown(hex_string))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_raw_signature() {
        let mut mac = Hmac::<Sha1>::new_from_slice(b"u9kqCo7qhKIQ8RML9xUGNmcZLVWmS8OsR2UN9jsZuaCY3aqPKGENRWmA36f9r47FHnqXlKuMvgsl0hnft7qCAN8iXHw94nHS4D6dxA07BX1lUjwuMJ0t73Z9wJY25Mpu")
            .expect("Can't create slice key!");

        mac.update(b"f0e5d42369441879d7e176c96cbbff2d");

        let concat_string = mac
            .finalize()
            .into_bytes()
            .iter()
            .map(|&byte| format!("{:02x}", byte))
            .collect::<String>();

        assert_eq!(concat_string, "26f59cab4eab972ec7dacec39a4355a3d7627717");
    }

    #[test]
    fn test_signature_struct() {
        let sign = Signature::new("u9kqCo7qhKIQ8RML9xUGNmcZLVWmS8OsR2UN9jsZuaCY3aqPKGENRWmA36f9r47FHnqXlKuMvgsl0hnft7qCAN8iXHw94nHS4D6dxA07BX1lUjwuMJ0t73Z9wJY25Mpu".to_string());

        assert_eq!(
            sign.get_digest("f0e5d42369441879d7e176c96cbbff2d"),
            "26f59cab4eab972ec7dacec39a4355a3d7627717"
        );
    }

    #[test]
    fn test_signature_get_key() {
        let sign_key = "u9kqCo7qhKIQ8RML9xUGNmcZLVWmS8OsR2UN9jsZuaCY3aqPKGENRWmA36f9r47FHnqXlKuMvgsl0hnft7qCAN8iXHw94nHS4D6dxA07BX1lUjwuMJ0t73Z9wJY25Mpu".to_string();
        let sign = Signature::new(sign_key.clone());
        assert_eq!(sign.key(), sign_key);
    }

    #[test]
    fn test_create_default() {
        let parser = Parser::default();
        assert!(!parser.check_key);
    }

    #[test]
    fn test_create_raw() {
        let parser = Parser::new(false, Signature::default());
        assert!(!parser.check_key);
    }

    #[test]
    fn test_signature_check_on() {
        let mut parser = Parser::new(false, Signature::default());
        assert!(!parser.check_key);
        parser.signature_check_on();
        assert!(parser.check_key);
    }

    #[test]
    fn test_signature_check_off() {
        let mut parser = Parser::new(true, Signature::default());
        assert!(parser.check_key);
        parser.signature_check_off();
        assert!(!parser.check_key);
    }

    #[test]
    fn test_set_signature_check() {
        let mut parser = Parser::new(true, Signature::default());
        assert_eq!(parser.get_key(), "");
        let new_signature = Signature::new("abc".to_string());
        parser.set_signature(new_signature);
        assert_eq!(parser.get_key(), "abc");
    }

    #[test]
    fn test_get_parser_status_disabled() {
        let parser = Parser::new(false, Signature::default());
        assert_eq!(parser.get_status(), "disabled");
    }

    #[test]
    fn test_get_parser_status_enabled() {
        let parser = Parser::new(true, Signature::new("abc".to_string()));
        assert_eq!(parser.get_status(), "enabled with key abc");
    }

    #[test]
    fn test_is_kind_for_private_channel() {
        let channel = Channel::create_private("abc".to_string());
        assert_eq!(channel.get_kind(), ChannelType::Private);
    }

    #[test]
    fn test_is_kind_for_public_channel() {
        let channel = Channel::create_public("abc".to_string());
        assert_eq!(channel.get_kind(), ChannelType::Public);
    }

    #[test]
    fn test_is_kind_for_unknown_channel() {
        let channel = Channel::create_unknown("abc".to_string());
        assert_eq!(channel.get_kind(), ChannelType::Unknown);
    }

    #[test]
    fn test_tostring_for_channel() {
        let prvt_channel = Channel::create_public("abc".to_string());
        assert_eq!(prvt_channel.to_string(), "abc");
    }

    #[test]
    fn test_empty_parse() {
        let parser = Parser::new(false, Signature::default());
        let parse_result = parser.parse("");
        assert!(parse_result.is_err());
        assert!(matches!(parse_result.unwrap_err(), ParseError::EmptyString));
    }

    #[test]
    fn test_parse_private_from_trusted() {
        let parser = Parser::new(false, Signature::default());
        let parse_result = parser.parse("3c8264bab589b0de7174e7b0523a40db");
        assert_eq!(
            parse_result.unwrap(),
            vec![Channel::create_private(
                "3c8264bab589b0de7174e7b0523a40db".to_string()
            )],
        );
    }

    #[test]
    fn test_parser_without_check_hash() {
        let parser = Parser::new(false, Signature::default());
        let parse_result = parser.parse("3c8264bab589b0de7174e7b0523a40db:c18beb389c3e49131dbb2dde597df615.fake_signature_string");
        assert_eq!(
            parse_result.unwrap(),
            vec![
                Channel::create_private("3c8264bab589b0de7174e7b0523a40db".to_string()),
                Channel::create_public("c18beb389c3e49131dbb2dde597df615".to_string()),
            ],
        );
    }

    #[test]
    fn test_parser_with_invalid_hasher() {
        let parser = Parser::new(true, Signature::new("wrong_code_in_advance".to_string()));
        let parse_result = parser.parse("3c8264bab589b0de7174e7b0523a40db:c18beb389c3e49131dbb2dde597df615.e4e4307e2c1485c9310f3a726c5af17ba380b828");
        assert!(parse_result.is_err());
        assert!(matches!(
            parse_result.unwrap_err(),
            ParseError::InvalidSignature(_)
        ));
    }

    #[test]
    fn test_parser_with_check_hash() {
        let parser = Parser::new(
            true,
            Signature::new("u9kqCo7qhKIQ8RML9xUGNmcZLVWmS8OsR2UN9jsZuaCY3aqPKGENRWmA36f9r47FHnqXlKuMvgsl0hnft7qCAN8iXHw94nHS4D6dxA07BX1lUjwuMJ0t73Z9wJY25Mpu".to_string()),
        );
        let parse_result = parser.parse("3c8264bab589b0de7174e7b0523a40db:c18beb389c3e49131dbb2dde597df615.e4e4307e2c1485c9310f3a726c5af17ba380b828");
        assert_eq!(
            parse_result.unwrap(),
            vec![
                Channel::create_private("3c8264bab589b0de7174e7b0523a40db".to_string()),
                Channel::create_public("c18beb389c3e49131dbb2dde597df615".to_string()),
            ],
        );
    }

    #[test]
    fn test_parser_multistring() {
        let parser = Parser::new(
            true,
            Signature::new("u9kqCo7qhKIQ8RML9xUGNmcZLVWmS8OsR2UN9jsZuaCY3aqPKGENRWmA36f9r47FHnqXlKuMvgsl0hnft7qCAN8iXHw94nHS4D6dxA07BX1lUjwuMJ0t73Z9wJY25Mpu".to_string()),
        );
        let parse_result = parser.parse("3c8264bab589b0de7174e7b0523a40db:c18beb389c3e49131dbb2dde597df615.e4e4307e2c1485c9310f3a726c5af17ba380b828/f0e5d42369441879d7e176c96cbbff2d.26f59cab4eab972ec7dacec39a4355a3d7627717");
        assert_eq!(
            parse_result.unwrap(),
            vec![
                Channel::create_private("3c8264bab589b0de7174e7b0523a40db".to_string()),
                Channel::create_public("c18beb389c3e49131dbb2dde597df615".to_string()),
                Channel::create_private("f0e5d42369441879d7e176c96cbbff2d".to_string()),
            ],
        );
    }

    #[test]
    fn test_malformed_channel_no_dot_with_check() {
        let parser = Parser::new(true, Signature::new("key".to_string()));
        let result = parser.parse("justdata");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ParseError::MalformedChannel(_)
        ));
    }

    #[test]
    fn test_success_get_channel_id_from_vec() {
        let channel_result = Channel::try_from(vec![
            130, 63, 11, 96, 124, 214, 171, 252, 114, 27, 229, 84, 157, 173, 240, 18,
        ]);
        assert!(channel_result.is_ok());
        assert_eq!(
            channel_result.unwrap().to_string(),
            "823f0b607cd6abfc721be5549dadf012"
        );
    }

    #[test]
    fn test_empty_get_channel_id_from_vec() {
        let channel_result = Channel::try_from(Vec::<u8>::new());
        assert!(channel_result.is_err());
        assert_eq!(channel_result.unwrap_err(), "Channel bytes is empty!");
    }

    #[test]
    fn test_parse_private_only_no_check() {
        let parser = Parser::new(false, Signature::default());
        let result = parser.parse("abc123").unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].to_string(), "abc123");
        assert_eq!(result[0].get_kind(), ChannelType::Private);
    }

    #[test]
    fn test_parse_three_part_data() {
        let parser = Parser::new(false, Signature::default());
        let result = parser.parse("a:b:c").unwrap();
        assert_eq!(result.len(), 2, "only first two parts used");
        assert_eq!(result[0].to_string(), "a");
        assert_eq!(result[1].to_string(), "b");
    }

    #[test]
    fn test_parse_trailing_slash() {
        let parser = Parser::new(false, Signature::default());
        let result = parser.parse("abc/").unwrap();
        assert_eq!(result.len(), 2, "trailing slash creates empty segment");
        assert_eq!(result[0].to_string(), "abc");
        assert_eq!(result[1].to_string(), "");
    }

    #[test]
    fn test_signature_deterministic() {
        let sign = Signature::new("key".to_string());
        let d1 = sign.get_digest("data");
        let d2 = sign.get_digest("data");
        assert_eq!(d1, d2);
    }

    #[test]
    fn test_signature_different_data_different_digest() {
        let sign = Signature::new("key".to_string());
        let d1 = sign.get_digest("data1");
        let d2 = sign.get_digest("data2");
        assert_ne!(d1, d2);
    }

    #[test]
    fn test_signature_empty_data() {
        let sign = Signature::new("key".to_string());
        let digest = sign.get_digest("");
        assert_eq!(digest.len(), 40, "SHA1 hex digest is 40 chars");
    }

    #[test]
    fn test_channel_display_private() {
        let ch = Channel::create_private("test123".to_string());
        assert_eq!(format!("{}", ch), "test123");
    }

    #[test]
    fn test_channel_try_from_single_byte() {
        let result = Channel::try_from(vec![0xff]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().to_string(), "ff");
    }
}
