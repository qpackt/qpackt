-- SPDX-License-Identifier: AGPL-3.0
--
--   qpackt: Web & Analytics Server
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

CREATE TABLE visits
(
    first_request_time INTEGER NOT NULL,
    last_request_time  INTEGER NOT NULL,
    request_count      INTEGER NOT NULL,
    visitor            INTEGER NOT NULL UNIQUE,
    version            TEXT    NOT NULL
);

CREATE INDEX visit_time_idx ON visits (first_request_time);
