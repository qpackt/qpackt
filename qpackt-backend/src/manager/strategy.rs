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

use serde::{Deserialize, Serialize};
/*
Deserialize to:
{"Weight":20.0}
{"UrlParam":"param"}
 */
/// Traffic split strategy.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[allow(variant_size_differences)]
pub(crate) enum Strategy {
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
    /// * version3 - weight 0
    /// version1 and version2 will get 50% of traffic. version3 will not get any traffic.
    Weight(u16),
    /// Matches new sessions that have url query containing the string.
    UrlParam(String),
}
