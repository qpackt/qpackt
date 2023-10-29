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

use serde::{Deserialize, Serialize};

/// Traffic split strategy.
#[derive(Debug, Serialize, Deserialize)]
#[allow(variant_size_differences)]
pub enum Strategy {
    /// No new traffic send to this version. Used as default for a new [Version]
    Inactive,
    /// Sends given percent of new sessions to corresponding [Version]
    Percent(f32),
    /// Sends new sessions that have given url param set to given value
    UrlParam(String, String),
}
