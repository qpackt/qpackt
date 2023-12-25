// SPDX-License-Identifier: AGPL-3.0
/*
   Vaden: Versioned Application Deployment Engine
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

use std::io;
use thiserror::Error;

pub(crate) type Result<T> = std::result::Result<T, VadenError>;

#[derive(Debug, Error)]
pub(crate) enum VadenError {
    #[error("invalid config {0}")]
    InvalidConfig(String),

    #[error("unable to read {0}")]
    UnableToRead(#[from] io::Error),

    #[error("unable to format {0}")]
    UnableToFormat(#[from] std::fmt::Error),

    #[error("unable to hash {0}")]
    UnableToHashPassword(#[from] scrypt::password_hash::Error),

    #[error("unable to read YAML {0}")]
    UnableToReadYaml(#[from] yaml_rust::ScanError),

    #[error("unable to proxy request")]
    ProxyError,

    // TODO change to 'from sqlx Error' for easier conversions
    #[error("unable to access database {0}")]
    DatabaseError(String),

    #[error("multipart upload error: {0}")]
    MultipartUploadError(String),

    #[error("site processing error: {0}")]
    UnableToProcessSite(String),
}
