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

function getVersion() {
    const cookieString = document.cookie
    const cookies = cookieString.split(';')

    for (let i = 0; i < cookies.length; i++) {
        const cookie = cookies[i].trim();
        const [cookieName, cookieValue] = cookie.split('=')

        if (cookieName === 'QPACKT_VERSION') {
            return decodeURIComponent(cookieValue)
        }
    }

    return ''
}

async function sendEvent(name, payload, visitor) {
    const version = getVersion()
    const xhr = new XMLHttpRequest();
    xhr.open("POST", '/qpackt/event', true);
    xhr.setRequestHeader('Content-Type', 'application/json');
    xhr.send(JSON.stringify({
        name,
        version,
        params: window.location.search,
        path: window.location.pathname,
        user_agent: window.navigator.userAgent,
        visitor,
        payload
    }));
}