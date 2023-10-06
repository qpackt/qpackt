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

use tokio::sync::RwLock;
use url::Url;

/// Represents a single upstream server to do proxy to.
pub(crate) struct Upstream {
    url: Url,
}

impl Upstream {
    #[allow(dead_code)] // TODO remove
    pub(crate) fn new(url: Url) -> Self {
        Self { url }
    }

    #[allow(dead_code)] // TODO remove
    pub(crate) fn url(&self) -> &Url {
        &self.url
    }
}

/// Encapsulation for multiple [Upstream]
#[derive(Default)]
pub(crate) struct Upstreams {
    upstreams: RwLock<Vec<Upstream>>,
}

impl Upstreams {
    pub(crate) async fn get_any_url(&self) -> Url {
        if let Ok(url) = std::env::var("UPSTREAM") {
            println!("proxying to {}", url);
            match Url::parse(&url) {
                Ok(url) => url,
                Err(e) => {
                    panic!("Unable to parse env upstream: {:?}", e);
                }
            }
        } else {
            self.upstreams.read().await[0].url.clone()
        }
    }
}
