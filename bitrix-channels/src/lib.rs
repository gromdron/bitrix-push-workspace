use hmac::{Hmac, Mac};
use sha1::Sha1;
use thiserror::Error;

#[allow(dead_code)]
#[derive(Clone, Default)]
pub struct Parser {
    check_key: bool,
    hasher: Signature,
}

pub type ChannelParseResult<T> = Result<T, ParseError>;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("String format error: {0}")]
    EmptyChannels(String),
    #[error("Channel strings is empty")]
    EmptyString,
}

impl Parser {
    pub fn new(check_key: bool, hasher: Signature) -> Parser {
        Parser { check_key, hasher }
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

    pub fn parse(&self, line: String) -> ChannelParseResult<Vec<Channel>> {
        if line.is_empty() {
            return Err(ParseError::EmptyString);
        }

        if line.len() == 32 {
            return Ok(vec![Channel::create_private(line)]);
        }

        let channels = line
            .split('/')
            .collect::<Vec<&str>>()
            .into_iter()
            .map(|channel_string| {
                let split_patrs = channel_string.split('.').collect::<Vec<&str>>();

                if split_patrs.len() != 2 {
                    return "";
                }

                if !self.check_key {
                    return split_patrs[0];
                }

                let calculated_hash = self.hasher.get_digest(split_patrs[0].to_string());

                if calculated_hash == split_patrs[1] {
                    return split_patrs[0];
                }

                ""
            })
            .filter(|channel_part| !channel_part.is_empty())
            .flat_map(|decoded_string| {
                let mut channels: Vec<Channel> = Vec::new();

                let parsed_channels = decoded_string
                    .split(':')
                    .map(|v| v.to_string())
                    .collect::<Vec<String>>();

                let mut parsed_channel_iter = parsed_channels.iter();

                if let Some(chnl) = parsed_channel_iter.next() {
                    channels.push(Channel::create_private(chnl.clone()));
                }

                if let Some(chnl) = parsed_channel_iter.next() {
                    channels.push(Channel::create_public(chnl.clone()));
                }

                channels
            })
            .collect::<Vec<Channel>>();

        Ok(channels)
    }

    pub fn get_key(&self) -> String {
        self.hasher.get_key()
    }

    pub fn get_status(&self) -> String {
        match self.check_key {
            true  => format!("enabled with key {}", self.hasher.get_key()),
            false => "disabled".to_string(),
        }
    }
}

#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct Signature {
    key: String,
}

impl Signature {
    pub fn new(key: String) -> Signature {
        Signature { key }
    }

    pub fn get_digest(&self, data: String) -> String {
        let mut mac = Hmac::<Sha1>::new_from_slice(&self.key.clone().into_bytes())
            .expect("Can't create slice key!");

        mac.update(&data.into_bytes());

        mac.finalize()
            .into_bytes()
            .into_iter()
            .map(|byte| format!("{:02x?}", byte))
            .collect::<String>()
    }

    pub fn get_key(&self) -> String {
        self.key.clone()
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
    pub fn create_private(channel_number: String) -> Channel {
        Channel {
            kind: ChannelType::Private,
            number: channel_number,
        }
    }

    pub fn create_public(channel_number: String) -> Channel {
        Channel {
            kind: ChannelType::Public,
            number: channel_number,
        }
    }

    pub fn create_unknown(channel_number: String) -> Channel {
        Channel {
            kind: ChannelType::Unknown,
            number: channel_number,
        }
    }

    pub fn get_kind(&self) -> ChannelType {
        self.kind.clone()
    }
}

impl ToString for Channel {
    fn to_string(&self) -> String {
        self.number.clone()
    }
}

impl TryFrom<Vec<u8>> for Channel {
    type Error = &'static str;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Err("Channel bytes is empty!");
        }

        let parsed = value
            .iter()
            .map(|n| format!("{:02x}", n))
            .fold(String::new(), |mut a, b| {
                a.push_str(&b);
                a
            });

        if parsed.is_empty() {
            return Err("Parsed bytes is empty after decoding");
        }

        Ok(Channel::create_unknown(parsed))
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
            .into_iter()
            .map(|byte| format!("{:02x?}", byte))
            .collect::<String>();

        assert_eq!(
            concat_string,
            "26f59cab4eab972ec7dacec39a4355a3d7627717".to_string()
        );
    }

    #[test]
    fn test_signature_struct() {
        let sign = Signature::new("u9kqCo7qhKIQ8RML9xUGNmcZLVWmS8OsR2UN9jsZuaCY3aqPKGENRWmA36f9r47FHnqXlKuMvgsl0hnft7qCAN8iXHw94nHS4D6dxA07BX1lUjwuMJ0t73Z9wJY25Mpu".to_string());

        assert_eq!(
            sign.get_digest("f0e5d42369441879d7e176c96cbbff2d".to_string()),
            "26f59cab4eab972ec7dacec39a4355a3d7627717".to_string()
        );
    }

    #[test]
    fn test_signature_get_key() {
        let sign_key = "u9kqCo7qhKIQ8RML9xUGNmcZLVWmS8OsR2UN9jsZuaCY3aqPKGENRWmA36f9r47FHnqXlKuMvgsl0hnft7qCAN8iXHw94nHS4D6dxA07BX1lUjwuMJ0t73Z9wJY25Mpu".to_string();
        let sign = Signature::new(sign_key.clone());

        assert_eq!(sign.get_key(), sign_key);
    }

    #[test]
    fn test_create_default() {
        let parser = Parser::default();

        assert!(!parser.check_key)
    }

    #[test]
    fn test_create_raw() {
        let parser = Parser::new(false, Signature::default());

        assert!(!parser.check_key);

        let parser = Parser::new(true, Signature::default());

        assert!(parser.check_key);
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

        let new_signature = Signature::new("abc".to_string());

        assert_eq!(parser.get_key(), "".to_string());

        parser.set_signature(new_signature);

        assert_eq!(parser.get_key(), "abc".to_string());
    }


    #[test]
    fn test_get_parser_status_disabled() {
        let parser = Parser::new(false, Signature::default());

        assert_eq!(parser.get_status(), "disabled".to_string());
    }

    #[test]
    fn test_get_parser_status_enabled() {
        let parser = Parser::new(true, Signature::new("abc".to_string()));

        assert_eq!(parser.get_status(), "enabled with key abc".to_string());
    }

    #[test]
    fn test_is_kind_for_private_channel() {
        let channel = Channel::create_private("abc".to_string());

        assert_eq!(channel.get_kind(), ChannelType::Private)
    }

    #[test]
    fn test_is_kind_for_public_channel() {
        let channel = Channel::create_public("abc".to_string());

        assert_eq!(channel.get_kind(), ChannelType::Public)
    }

    #[test]
    fn test_is_kind_for_unknown_channel() {
        let channel = Channel::create_unknown("abc".to_string());

        assert_eq!(channel.get_kind(), ChannelType::Unknown)
    }

    #[test]
    fn test_tostring_for_channel() {
        let prvt_channel = Channel::create_public("abc".to_string());

        assert_eq!(prvt_channel.to_string(), "abc".to_string())
    }

    #[test]
    fn test_parse_private_channel() {
        let sign = Signature::new( "u9kqCo7qhKIQ8RML9xUGNmcZLVWmS8OsR2UN9jsZuaCY3aqPKGENRWmA36f9r47FHnqXlKuMvgsl0hnft7qCAN8iXHw94nHS4D6dxA07BX1lUjwuMJ0t73Z9wJY25Mpu".to_string() );

        let channel_string = String::from("f0e5d42369441879d7e176c96cbbff2d");

        assert_eq!(
            sign.get_digest(channel_string),
            "26f59cab4eab972ec7dacec39a4355a3d7627717".to_string()
        );
    }

    #[test]
    fn test_parse_mixed_channel() {
        let sign = Signature::new( "u9kqCo7qhKIQ8RML9xUGNmcZLVWmS8OsR2UN9jsZuaCY3aqPKGENRWmA36f9r47FHnqXlKuMvgsl0hnft7qCAN8iXHw94nHS4D6dxA07BX1lUjwuMJ0t73Z9wJY25Mpu".to_string() );

        let channel_string =
            String::from("3c8264bab589b0de7174e7b0523a40db:c18beb389c3e49131dbb2dde597df615");

        assert_eq!(
            sign.get_digest(channel_string),
            "e4e4307e2c1485c9310f3a726c5af17ba380b828".to_string()
        );
    }

    #[test]
    fn test_empty_parse() {
        let parser = Parser::new(false, Signature::default());

        let string_to_parse = String::from("");

        let parse_result = parser.parse(string_to_parse);

        assert!(parse_result.is_err())
    }

    #[test]
    fn test_parse_private_from_trusted() {
        let parser = Parser::new(false, Signature::default());

        let string_to_parse = String::from("3c8264bab589b0de7174e7b0523a40db");

        let parse_result = parser.parse(string_to_parse);

        assert_eq!(
            parse_result.unwrap(),
            vec![Channel::create_private(
                "3c8264bab589b0de7174e7b0523a40db".to_string()
            )],
        )
    }

    #[test]
    fn test_parser_without_check_hash() {
        let parser = Parser::new(false, Signature::default());

        let string_to_parse = String::from("3c8264bab589b0de7174e7b0523a40db:c18beb389c3e49131dbb2dde597df615.fake_signature_string");

        let parse_result = parser.parse(string_to_parse);

        assert_eq!(
            parse_result.unwrap(),
            vec![
                Channel::create_private("3c8264bab589b0de7174e7b0523a40db".to_string()),
                Channel::create_public("c18beb389c3e49131dbb2dde597df615".to_string()),
            ],
        )
    }

    #[test]
    fn test_parser_with_invalid_hasher() {
        let parser = Parser::new(true, Signature::new("wrong_code_in_advance".to_string()));

        let string_to_parse = String::from("3c8264bab589b0de7174e7b0523a40db:c18beb389c3e49131dbb2dde597df615.e4e4307e2c1485c9310f3a726c5af17ba380b828");

        let parse_result = parser.parse(string_to_parse);

        assert_eq!(parse_result.unwrap(), vec![],)
    }

    #[test]
    fn test_parser_with_check_hash() {
        let parser = Parser::new(true, Signature::new("u9kqCo7qhKIQ8RML9xUGNmcZLVWmS8OsR2UN9jsZuaCY3aqPKGENRWmA36f9r47FHnqXlKuMvgsl0hnft7qCAN8iXHw94nHS4D6dxA07BX1lUjwuMJ0t73Z9wJY25Mpu".to_string()));

        let string_to_parse = String::from("3c8264bab589b0de7174e7b0523a40db:c18beb389c3e49131dbb2dde597df615.e4e4307e2c1485c9310f3a726c5af17ba380b828");

        let parse_result = parser.parse(string_to_parse);

        assert_eq!(
            parse_result.unwrap(),
            vec![
                Channel::create_private("3c8264bab589b0de7174e7b0523a40db".to_string()),
                Channel::create_public("c18beb389c3e49131dbb2dde597df615".to_string()),
            ],
        )
    }

    #[test]
    fn test_parser_multistring() {
        let parser = Parser::new(true, Signature::new("u9kqCo7qhKIQ8RML9xUGNmcZLVWmS8OsR2UN9jsZuaCY3aqPKGENRWmA36f9r47FHnqXlKuMvgsl0hnft7qCAN8iXHw94nHS4D6dxA07BX1lUjwuMJ0t73Z9wJY25Mpu".to_string()));

        let string_to_parse = String::from("3c8264bab589b0de7174e7b0523a40db:c18beb389c3e49131dbb2dde597df615.e4e4307e2c1485c9310f3a726c5af17ba380b828/f0e5d42369441879d7e176c96cbbff2d.26f59cab4eab972ec7dacec39a4355a3d7627717");

        let parse_result = parser.parse(string_to_parse);

        assert_eq!(
            parse_result.unwrap(),
            vec![
                Channel::create_private("3c8264bab589b0de7174e7b0523a40db".to_string()),
                Channel::create_public("c18beb389c3e49131dbb2dde597df615".to_string()),
                Channel::create_private("f0e5d42369441879d7e176c96cbbff2d".to_string()),
            ],
        )
    }

    #[test]
    fn test_success_get_channel_id_from_vec() {
        let channel_result = Channel::try_from(vec![
            130, 63, 11, 96, 124, 214, 171, 252, 114, 27, 229, 84, 157, 173, 240, 18,
        ]);

        assert!(channel_result.is_ok());

        assert_eq!(
            channel_result.unwrap().to_string(),
            String::from("823f0b607cd6abfc721be5549dadf012")
        )
    }

    #[test]
    fn test_success_get_channel_id_from_vec_2() {
        let channel_result = Channel::try_from(vec![
            85, 127, 119, 97, 129, 227, 186, 90, 154, 40, 219, 73, 142, 65, 241, 59,
        ]);

        assert!(channel_result.is_ok());

        assert_eq!(
            channel_result.unwrap().to_string(),
            String::from("557f776181e3ba5a9a28db498e41f13b")
        )
    }

    #[test]
    fn test_empty_get_channel_id_from_vec() {
        let channel_result = Channel::try_from(Vec::new());

        assert!(channel_result.is_err());
    }
}
