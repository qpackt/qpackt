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

use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

pub(super) struct DaoInner {
    rw_url: RwLock<String>,
    ro_url: RwLock<String>,
}

impl DaoInner {
    /// Sqlite doesn't not support concurrent writes. To prevent simultaneous , writes are possible only when
    /// write lock to rw_url is held.
    pub(super) fn init(path: &str) -> Self {
        let rw_url = format!("sqlite://{path}?mode=rwc");
        let ro_url = format!("sqlite://{path}?mode=ro");
        Self { rw_url: RwLock::new(rw_url), ro_url: RwLock::new(ro_url) }
    }
    pub(super) async fn get_read_only_url(&self) -> RwLockReadGuard<'_, String> {
        self.ro_url.read().await
    }

    pub(super) async fn get_read_write_url(&self) -> RwLockWriteGuard<'_, String> {
        self.rw_url.write().await
    }
}
