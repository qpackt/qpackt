-- SPDX-License-Identifier: AGPL-3.0
--
--   Vaden: Versioned Application Deployment Engine
--   Copyright (C) 2023 Łukasz Wojtów

--   This program is free software: you can redistribute it and/or modify
--   it under the terms of the GNU Affero General Public License as
--   published by the Free Software Foundation, either version 3 of the
--   License.

--   This program is distributed in the hope that it will be useful,
--   but WITHOUT ANY WARRANTY; without even the implied warranty of
--   MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
--   GNU Affero General Public License for more details.

--   You should have received a copy of the GNU Affero General Public License
--   along with this program.  If not, see <https://www.gnu.org/licenses/>.

CREATE TABLE versions (
    web_root TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL UNIQUE
);

CREATE TABLE settings (
    name TEXT NOT NULL UNIQUE,
    value JSON NOT NULL
)