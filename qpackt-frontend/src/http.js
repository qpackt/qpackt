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

import axios from 'axios'
import {getToken} from "./state.js";

export const http = {
    axiosInstance: axios.create(),

    getConfig() {
        let config = {
            headers: {}
        }
        let token = getToken();
        if (token !== 0) {
            config.headers['Authorization'] = `Bearer ${token}`
        }
        return config
    },

    async get(endpoint) {
        const config = this.getConfig()
        return await this.axiosInstance.get(endpoint, config)
    },

    async post(endpoint, data) {
        const config = this.getConfig()
        return await this.axiosInstance.post(endpoint, data, config)
    },

    async put(endpoint, data) {
        const config = this.getConfig()
        return await this.axiosInstance.put(endpoint, data, config)
    },

    async delete(endpoint) {
        const config = this.getConfig()
        return await this.axiosInstance.delete(endpoint, config)
    },

    async downloadCsv(url, filename) {
        const config = this.getConfig();
        config.responseType = 'blob';
        axios.get(url, config)
            .then(response => {
                const url = window.URL.createObjectURL(new Blob([response.data]));
                const link = document.createElement('a');
                link.href = url;
                link.setAttribute('download', filename);
                document.body.appendChild(link);
                link.click();
                document.body.removeChild(link);
            })
            .catch(error => {
                console.error('There was a problem with the fetch operation:', error);
            });
    }
}
