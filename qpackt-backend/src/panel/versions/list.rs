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

use crate::dao::Dao;
use crate::error::Result;
use crate::panel::validate_permission;
use actix_web::web::{Data, Json};
use actix_web::{HttpRequest, Responder};
use log::error;

pub(crate) async fn list_versions(request: HttpRequest, dao: Data<Dao>) -> Result<impl Responder> {
    validate_permission(&request)?;
    match dao.list_versions().await {
        Ok(versions) => Ok(Json(versions)),
        Err(e) => {
            error!("Unable to list versions: {}", e.to_string());
            Err(e)
        }
    }
}
