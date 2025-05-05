use anyhow::{anyhow, Result};
use argon2::password_hash::{rand_core::OsRng, PasswordHasher, SaltString};
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use shaku::Interface;

pub trait PasswordHandlerLike: Interface + Send + Sync {
    fn validate_password_against_mcf(
        &self,
        mcf_string_like: &impl AsRef<str>,
        provided_password: &impl AsRef<str>,
    ) -> Result<()>;

    fn generate_password_hash(&self, provided_password: &impl AsRef<str>) -> Result<String>;
}

impl PasswordHandlerLike for Argon2<'static> {
    fn validate_password_against_mcf(
        &self,
        mcf_string_like: &impl AsRef<str>,
        provided_password: &impl AsRef<str>,
    ) -> Result<()> {
        let password_bytes = provided_password.as_ref().as_bytes();
        let mcf_string = mcf_string_like.as_ref();

        // 1. Parse the stored hash string.
        //    PasswordHash::new requires the hash string itself.
        let parsed_hash = PasswordHash::new(mcf_string)
            .map_err(|_| anyhow!("Couldn't hash passwod".to_string()))?;

        // 2. Verify the password.
        //    The `verify_password` method is part of the `PasswordVerifier` trait,
        //    implemented by Argon2. It takes the password bytes and the parsed hash.
        self.verify_password(password_bytes, &parsed_hash)
            .map_err(|_| anyhow!("Passwords don't match".to_string()))
    }

    fn generate_password_hash(&self, provided_password: &impl AsRef<str>) -> Result<String> {
        let salt = SaltString::generate(&mut OsRng);
        self.hash_password(provided_password.as_ref().as_bytes(), &salt)
            .map(|v| v.to_string())
            .map_err(|e| anyhow!(e))
    }
}
