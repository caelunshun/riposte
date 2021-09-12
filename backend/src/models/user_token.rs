use bytes::Bytes;
use rand::{rngs::OsRng, Rng};
use subtle::ConstantTimeEq;
use uuid::Uuid;

const TOKEN_LEN: usize = 32;

#[derive(Debug, Clone)]
pub struct UserAccessToken {
    token: Bytes,
    user_id: Uuid,
}

impl UserAccessToken {
    pub fn new(token: Bytes, user_id: Uuid) -> Self {
        Self { token, user_id }
    }

    pub fn generate_for_user(user_id: Uuid) -> Self {
        let token: [u8; TOKEN_LEN] = OsRng.gen();
        Self {
            token: token.to_vec().into(),
            user_id,
        }
    }

    pub fn token_bytes(&self) -> &[u8] {
        &self.token
    }

    pub fn user_id(&self) -> Uuid {
        self.user_id
    }

    #[allow(dead_code)]
    pub fn matches(&self, token: &[u8]) -> bool {
        token.ct_eq(self.token_bytes()).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_tokens_are_unique() {
        let user_id = Uuid::new_v4();
        let token1 = UserAccessToken::generate_for_user(user_id);
        let token2 = UserAccessToken::generate_for_user(user_id);
        assert_ne!(token1.token_bytes(), token2.token_bytes());
    }
}
