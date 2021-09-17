use argon2::{password_hash::SaltString, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use bytes::Bytes;
use rand::rngs::OsRng;
use uuid::Uuid;

#[derive(Debug)]
pub struct User {
    id: Uuid,
    username: String,
    email: String,
    hashed_password: Bytes,
}

impl User {
    pub fn new(id: Uuid, username: String, email: String, hashed_password: Bytes) -> Self {
        Self {
            id,
            username,
            email,
            hashed_password,
        }
    }

    pub fn create(
        username: String,
        email: String,
        password: String,
    ) -> Result<Self, UserCreateError> {
        validate_username(&username)?;
        validate_password(&password)?;

        let id = Uuid::new_v4();
        let hashed_password = hash_password(&password);

        Ok(Self {
            id,
            username,
            email,
            hashed_password,
        })
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn email(&self) -> &str {
        &self.email
    }

    pub fn hashed_password(&self) -> &[u8] {
        &self.hashed_password
    }

    pub fn check_password(&self, password: &str) -> Result<(), WrongPassword> {
        check_password(&self.hashed_password, password)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum UserCreateError {
    #[error("invalid username")]
    InvalidUsername,
    #[error("invalid password")]
    InvalidPassword,
}

const MIN_USERNAME_LEN: usize = 4;
const MAX_USERNAME_LEN: usize = 32;

fn validate_username(username: &str) -> Result<(), UserCreateError> {
    for c in username.chars() {
        if !c.is_ascii_alphanumeric() && c != '_' {
            return Err(UserCreateError::InvalidUsername);
        }
    }

    if !(MIN_USERNAME_LEN..=MAX_USERNAME_LEN).contains(&username.len()) {
        return Err(UserCreateError::InvalidPassword);
    }

    Ok(())
}

const MIN_PASSWORD_LEN: usize = 8;

fn validate_password(password: &str) -> Result<(), UserCreateError> {
    if password.len() < MIN_PASSWORD_LEN {
        Err(UserCreateError::InvalidPassword)
    } else {
        Ok(())
    }
}

fn hash_password(password: &str) -> Bytes {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password_simple(password.as_bytes(), &salt)
        .expect("failed to hash password");
    hash.to_string().into()
}

#[derive(Debug, thiserror::Error)]
#[error("password is incorrect")]
pub struct WrongPassword;

fn check_password(hashed_password: &[u8], password: &str) -> Result<(), WrongPassword> {
    let hash = PasswordHash::new(std::str::from_utf8(hashed_password).map_err(|_| WrongPassword)?)
        .map_err(|_| WrongPassword)?;
    let argon2 = Argon2::default();
    argon2
        .verify_password(password.as_bytes(), &hash)
        .map_err(|_| WrongPassword)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_create_hashes_password() {
        let password = "password".to_owned();
        let user = User::create(
            "caelunshun".to_owned(),
            "caelunshun@gmail.com".to_owned(),
            password.clone(),
        )
        .unwrap();
        assert_ne!(user.hashed_password, password.as_bytes());
    }

    #[test]
    fn validate_username_alphabetical() {
        assert!(validate_username("AbcD").is_ok());
    }

    #[test]
    fn validate_username_numeric() {
        assert!(validate_username("1234").is_ok());
    }

    #[test]
    fn validate_username_underscore() {
        assert!(validate_username("___Chr__onicbeard").is_ok());
    }

    #[test]
    fn validate_username_invalid_symbols() {
        assert!(validate_username("The Way%5").is_err());
        assert!(validate_username("-- Hyphen world").is_err());
        assert!(validate_username("$500.50").is_err());
    }

    #[test]
    fn validate_password_too_short() {
        assert!(validate_password("abcdefg").is_err());
    }

    #[test]
    fn validate_password_okay() {
        assert!(validate_password("!@#$%^&*").is_ok());
    }

    #[test]
    fn validate_username_too_short() {
        assert!(validate_username("abc").is_err());
        assert!(validate_username("a").is_err());
        assert!(validate_username("").is_err());
    }

    #[test]
    fn validate_username_too_long() {
        assert!(validate_username("More than 32 characters is a bad ").is_err());
    }

    #[test]
    fn user_create_checks_username() {
        assert!(
            User::create("abc".to_owned(), "efw".to_owned(), "erorihhieog".to_owned()).is_err()
        );
        assert!(User::create(
            "abcd".to_owned(),
            "efw".to_owned(),
            "erorihhieog".to_owned()
        )
        .is_ok());
    }

    #[test]
    fn user_create_checks_password() {
        assert!(User::create(
            "caelunshun".to_owned(),
            "email".to_owned(),
            "short".to_owned()
        )
        .is_err());
        assert!(User::create(
            "caelunshun".to_owned(),
            "email".to_owned(),
            "nice and long".to_owned()
        )
        .is_ok());
    }

    #[test]
    fn hash_and_check_password() {
        let password = "thePassword".to_owned();
        let hashed = hash_password(&password);
        assert!(check_password(&hashed, &password).is_ok());
        assert!(check_password(&hashed, "wrongPassword").is_err());
    }

    #[test]
    fn hashes_are_salted() {
        let password = "thePassword".to_owned();
        let hashed1 = hash_password(&password);
        let hashed2 = hash_password(&password);
        assert_ne!(hashed1, hashed2);
    }
}
