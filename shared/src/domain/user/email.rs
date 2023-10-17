use std::str::FromStr;

use validator::validate_email;

use crate::domain;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct UserEmail(String);

impl FromStr for UserEmail {
    type Err = domain::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.to_owned().try_into()
    }
}

impl TryFrom<String> for UserEmail {
    type Error = domain::Error;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        if validate_email(&s) {
            Ok(Self(s))
        } else {
            Err(domain::Error::ValidationError(format!(
                "{} is not a valid subscriber email.",
                s
            )))
        }
    }
}

impl AsRef<str> for UserEmail {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::UserEmail;
    use claims::assert_err;
    use fake::faker::internet::en::SafeEmail;
    use fake::Fake;
    use rand::prelude::StdRng;
    use rand::SeedableRng;

    #[test]
    fn empty_string_is_rejected() {
        assert_err!(UserEmail::from_str(""));
    }

    #[test]
    fn email_missing_at_symbol_is_rejected() {
        assert_err!(UserEmail::from_str("ursuladomain.com"));
    }

    #[test]
    fn email_missing_subject_is_rejected() {
        assert_err!(UserEmail::from_str("@domain.com"));
    }

    #[derive(Debug, Clone)]
    struct ValidEmailFixture(pub String);

    impl quickcheck::Arbitrary for ValidEmailFixture {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            let mut rng = StdRng::seed_from_u64(u64::arbitrary(g));
            let email = SafeEmail().fake_with_rng(&mut rng);
            Self(email)
        }
    }

    #[quickcheck_macros::quickcheck]
    fn valid_emails_are_parsed_successfully(valid_email: ValidEmailFixture) -> bool {
        UserEmail::from_str(&valid_email.0).is_ok()
    }
}
