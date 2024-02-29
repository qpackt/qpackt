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

use actix_web::http::Uri;
use arc_swap::ArcSwap;
use std::sync::Arc;

#[derive(Clone)]
pub(crate) struct ReverseProxy {
    pub(crate) id: i32,
    pub(crate) prefix: String,
    pub(crate) target: String,
}

#[derive(Default)]
pub(crate) struct ReverseProxies {
    list: ArcSwap<Vec<ReverseProxy>>,
}

impl ReverseProxies {
    pub(crate) async fn set(&self, list: Vec<ReverseProxy>) {
        let new_list = Arc::new(list);
        self.list.store(new_list);
    }

    pub(crate) fn find_by_uri(&self, uri: &Uri) -> Option<ReverseProxy> {
        let list = self.list.load();
        for rp in list.iter() {
            if uri.path().starts_with(&rp.prefix) {
                return Some(rp.clone());
            }
        }
        None
    }
}
