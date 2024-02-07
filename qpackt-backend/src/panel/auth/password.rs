// SPDX-License-Identifier: AGPL-3.0
/*
   qpackt: Web & Analytics Server
   Copyright (C) 2023 Łukasz Wojtów

   This program is free software: you can redistribute it and/or modify
   it under the terms of the GNU Affero General Public License as
   published by the Free Software Foundation, either version 3 of the
   License.

   This program is distributed in the hope that it will be useful,
   but WITHOUT ANY WARRANTY; without even the implied warranty of
   MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
   GNU Affero General Public License for more details.

   You should have received a copy of the GNU Affero General Public License
   along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

use crate::error::{QpacktError, Result};

use scrypt::password_hash::{PasswordHash, PasswordVerifier};
use scrypt::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Scrypt,
};

pub(crate) fn hash_password(password: String) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let hash = Scrypt.hash_password(password.as_bytes(), &salt)?.to_string();
    Ok(hash)
}

/// Checks if password provided by the user (`password` arg) matches the `hash` read from the database.
/// Returns Ok(true/false) when able to check, Error otherwise.
pub(crate) fn password_matches(password: String, hash: &str) -> Result<bool> {
    let hash = PasswordHash::new(hash).map_err(QpacktError::UnableToHashPassword)?;
    let result = Scrypt.verify_password(password.as_bytes(), &hash);
    if result.is_ok() {
        Ok(true)
    } else {
        Ok(false)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn matches() {
        let password = "Pass";
        let hash = hash_password(password.to_string()).unwrap();
        let matches = password_matches(password.to_string(), &hash).unwrap();
        assert!(matches);
    }

    #[test]
    fn doesnt_match() {
        let hash = hash_password("Pass".to_string()).unwrap();
        let matches = password_matches("OtherPass".to_string(), &hash).unwrap();
        assert!(!matches);
    }
}
