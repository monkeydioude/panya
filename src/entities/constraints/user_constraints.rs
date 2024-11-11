use super::constraints::Constraint;
use crate::{entities::user::User, error::Error};

pub struct UserConstraints<'a>(&'a User);
use regex::Regex;

impl<'a> UserConstraints<'a> {
    fn check_username(&self) -> Result<bool, Error> {
        Ok(
            Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$")
                .map_err(|err| Error::from(err))?
                .is_match(&self.0.username),
        )
    }
}

impl<'a> Constraint<User> for UserConstraints<'a> {
    fn assert(&self) -> Result<bool, Error> {
        self.check_username()
    }
}

#[cfg(test)]
mod test {
    use crate::entities::{constraints::constraints::Constraint, user::User};

    use super::UserConstraints;

    #[test]
    fn test_i_can_assert_username() {
        let user1 = User {
            id: 1,
            username: "salut@test.com".to_string(),
            channel_ids: vec![],
        };
        assert_eq!(true, UserConstraints(&user1).assert().unwrap());

        let user2 = User {
            id: 2,
            username: "salut".to_string(),
            channel_ids: vec![],
        };
        assert_eq!(false, UserConstraints(&user2).assert().unwrap());
    }
}
