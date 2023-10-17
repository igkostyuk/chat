use std::str::FromStr;
use unicode_segmentation::UnicodeSegmentation;

use crate::domain;

const MAX_MESSAGE_CONTENT_SIZE: usize = 255;

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct MessageContent(String);

impl FromStr for MessageContent {
    type Err = domain::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.to_owned().try_into()
    }
}

impl TryFrom<String> for MessageContent {
    type Error = domain::Error;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        let is_empty_or_whitespace = s.trim().is_empty();

        let is_too_long = s.graphemes(true).count() > MAX_MESSAGE_CONTENT_SIZE;

        if is_empty_or_whitespace || is_too_long {
            Err(domain::Error::ValidationError(format!(
                "{} is not a valid user code.",
                s
            )))
        } else {
            Ok(Self(s))
        }
    }
}

impl AsRef<str> for MessageContent {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod room_code_tests {
    use super::*;
    use claims::{assert_err, assert_ok};

    #[test]
    fn a_size_grapheme_long_is_valid() {
        let name = "a̐".repeat(MAX_MESSAGE_CONTENT_SIZE);
        assert_ok!(name.parse::<MessageContent>());
    }

    #[test]
    fn a_longer_than_size_graphemes_is_rejected() {
        let name = "a".repeat(MAX_MESSAGE_CONTENT_SIZE + 1);
        assert_err!(name.parse::<MessageContent>());
    }

    #[test]
    fn whitespace_only_are_rejected() {
        let name = " ".to_string();
        assert_err!(name.parse::<MessageContent>());
    }

    #[test]
    fn empty_string_is_rejected() {
        let name = "".to_string();
        assert_err!(name.parse::<MessageContent>());
    }

    #[test]
    fn a_valid_is_parsed_successfully() {
        let name = "Code".to_string();
        assert_ok!(name.parse::<MessageContent>());
    }
}
