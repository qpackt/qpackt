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
/*
Deserialize to:
"Inactive"
{"Weight":20.0}
{"UrlParam":["source","ad"]}
 */
/// Traffic split strategy.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[allow(variant_size_differences)]
pub enum Strategy {
    /// No new traffic send to this version. Used as default for a new [Version]
    Inactive,
    /// Calculate percent of sessions based on the total sum of all weights.
    /// Only 'Weight' strategies are counted towards the total sum.
    /// Example:
    /// * version1 - weight 1
    /// * version2 - weight 9
    /// * version3 - url param
    /// version2 will get 90% of traffic, version1 will get 10%, version3 will get all traffic with required url param.
    /// Example:
    /// * version1 - weight 10
    /// * version2 - weight 10
    /// * version3 - inactive
    /// version1 and version2 will get 50% of traffic. version3 will not get any traffic.
    Weight(f32), // TODO change to integer
    /// Sends new sessions that have given url param set to given value
    UrlParam(String, String), // TODO change to single String
}
