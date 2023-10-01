use crate::error::Result;

use scrypt::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Scrypt,
};

pub fn hash_password(password: String) -> Result<String> {
    println!("Hashing password...");
    let salt = SaltString::generate(&mut OsRng);
    let hash = Scrypt
        .hash_password(password.as_bytes(), &salt)?
        .to_string();
    println!("Done ashing");
    Ok(hash)
}
