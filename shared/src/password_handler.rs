use anyhow::{anyhow, Result};
use argon2::password_hash::{rand_core::OsRng, PasswordHasher, SaltString};
use argon2::{Argon2, PasswordHash, PasswordVerifier};

/// A trait for handling password hashing and verification.
///
/// This trait abstracts the specific password hashing algorithm used.
/// It requires `Send + Sync` to be safely used across threads.
pub trait PasswordHandlerLike: Send + Sync {
    /// Validates a provided password against a Modular Crypt Format (MCF) string.
    ///
    /// # Arguments
    ///
    /// * `mcf_string_like` - A reference to a string-like type containing the MCF hash.
    /// * `provided_password` - A reference to a string-like type containing the password to validate.
    ///
    /// # Errors
    ///
    /// Returns an error if the MCF string is invalid or if the password does not match the hash.
    fn validate_password_against_mcf(
        &self,
        mcf_string_like: &impl AsRef<str>,
        provided_password: &impl AsRef<str>,
    ) -> Result<()>;

    /// Generates a password hash (MCF string) for a given password.
    ///
    /// # Arguments
    ///
    /// * `provided_password` - A reference to a string-like type containing the password to hash.
    ///
    /// # Returns
    ///
    /// A `Result` containing the generated MCF string hash on success, or an error on failure.
    ///
    /// # Errors
    ///
    /// If for any reason the password couldn't be hashed.
    fn generate_password_hash(&self, provided_password: &impl AsRef<str>) -> Result<String>;
}

/// Implementation of `PasswordHandlerLike` using the Argon2 algorithm.
impl PasswordHandlerLike for Argon2<'static> {
    /// Validates a password against an Argon2 MCF hash.
    ///
    /// It parses the MCF string and then uses the `PasswordVerifier` implementation
    /// of `Argon2` to check the password.
    fn validate_password_against_mcf(
        &self,
        mcf_string_like: &impl AsRef<str>,
        provided_password: &impl AsRef<str>,
    ) -> Result<()> {
        let password_bytes = provided_password.as_ref().as_bytes();
        let mcf_string = mcf_string_like.as_ref();
        // Attempt to parse the hash string
        let parsed_hash = PasswordHash::new(mcf_string)
            .map_err(|e| anyhow!("Error parsing password hash: {}", e))?;

        // Verify the password against the parsed hash
        self.verify_password(password_bytes, &parsed_hash)
            .map_err(|e| anyhow!("Password verification failed: {}", e))
    }

    /// Generates an Argon2 password hash (MCF string).
    ///
    /// It generates a random salt using `OsRng` and then uses the `PasswordHasher`
    /// implementation of `Argon2` to create the hash.
    fn generate_password_hash(&self, provided_password: &impl AsRef<str>) -> Result<String> {
        let salt = SaltString::generate(&mut OsRng);
        self.hash_password(provided_password.as_ref().as_bytes(), &salt)
            .map(|hash| hash.to_string())
            .map_err(|e| anyhow!("Error generating password hash: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use argon2::Argon2; // Import Argon2 directly for instantiation

    // Helper function to get a default Argon2 instance for tests
    fn get_argon2_handler() -> Argon2<'static> {
        Argon2::default()
    }

    #[test]
    fn test_generate_password_hash_success() {
        let handler = get_argon2_handler();
        let password = "mysecretpassword";
        let result = handler.generate_password_hash(&password);

        assert!(result.is_ok());
        let hash = result.unwrap();
        // Basic check: Ensure the hash is not empty and looks like an Argon2 MCF string
        assert!(!hash.is_empty());
        assert!(hash.starts_with("$argon2"));
    }

    #[test]
    fn test_validate_password_success() {
        let handler = get_argon2_handler();
        let password = "mysecretpassword";
        let hash = handler.generate_password_hash(&password).unwrap();

        let result = handler.validate_password_against_mcf(&hash, &password);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_password_failure_wrong_password() {
        let handler = get_argon2_handler();
        let password = "mysecretpassword";
        let wrong_password = "wrongpassword";
        let hash = handler.generate_password_hash(&password).unwrap();

        let result = handler.validate_password_against_mcf(&hash, &wrong_password);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Password verification failed"));
    }

    #[test]
    fn test_validate_password_failure_invalid_hash() {
        let handler = get_argon2_handler();
        let password = "mysecretpassword";
        let invalid_hash = "not_a_valid_mcf_string";

        let result = handler.validate_password_against_mcf(&invalid_hash, &password);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Error parsing password hash"));
    }
}
