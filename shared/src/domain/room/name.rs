use std::str::FromStr;
use unicode_segmentation::UnicodeSegmentation;

use crate::domain;
use crate::domain::utils::contains_forbidden_characters;

const MAX_ROOM_NAME_SIZE: usize = 255;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct RoomName(String);

impl FromStr for RoomName {
    type Err = domain::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.to_owned().try_into()
    }
}

impl TryFrom<String> for RoomName {
    type Error = domain::Error;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        let is_empty_or_whitespace = s.trim().is_empty();

        let is_too_long = s.graphemes(true).count() > MAX_ROOM_NAME_SIZE;

        if is_empty_or_whitespace || is_too_long || contains_forbidden_characters(&s) {
            Err(domain::Error::ValidationError(format!(
                "{} is not a valid subscriber name.",
                s
            )))
        } else {
            Ok(Self(s))
        }
    }
}

impl AsRef<str> for RoomName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod room_name_tests {
    use super::*;
    use claims::{assert_err, assert_ok};

    #[test]
    fn a_size_grapheme_long_name_is_valid() {
        let name = "a̐".repeat(MAX_ROOM_NAME_SIZE);
        assert_ok!(name.parse::<RoomName>());
    }

    #[test]
    fn a_name_longer_than_size_graphemes_is_rejected() {
        let name = "a".repeat(MAX_ROOM_NAME_SIZE + 1);
        assert_err!(name.parse::<RoomName>());
    }

    #[test]
    fn whitespace_only_names_are_rejected() {
        let name = " ".to_string();
        assert_err!(name.parse::<RoomName>());
    }

    #[test]
    fn empty_string_is_rejected() {
        let name = "".to_string();
        assert_err!(name.parse::<RoomName>());
    }

    #[test]
    fn names_containing_an_invalid_character_are_rejected() {
        for name in &['/', '(', ')', '"', '<', '>', '\\', '{', '}'] {
            let name = name.to_string();
            assert_err!(name.parse::<RoomName>());
        }
    }

    #[test]
    fn a_valid_name_is_parsed_successfully() {
        let name = "Ursula".to_string();
        assert_ok!(name.parse::<RoomName>());
    }
}
