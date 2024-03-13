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

import {reactive} from 'vue';

/**
 * This is application state. Implementation provides the same functionality as vuex or redux but with slightly
 * better api. Components are not supposed to have own data but use getters instead.
 * Also, state should not be accessed directly from components. Instead, modification functions should be called.
 * Each component has its own 'substate' to keep things organized.
 * See examples in components for usage.
 *
 */
const state = reactive({
    /**
     * Token for logged in user.
     */
    authorization: {
        token: '',
    },
    /**
     * /analytics page state
     */
    analytics: {
        dateStart: {},
        dateEnd: {},
        totalVisits: 0,
        stats: [],
        events: {
            /**
             * Vec of event stats. Single item is:
             * {
             *             "event": "TIME_ON_PAGE_7",
             *             "percents": [
             *                 {
             *                     "version": "2024_03_13__09_49_32",
             *                     "percent": 0.0
             *                 },
             *                 {
             *                     "version": "2024_03_16__14_17_39",
             *                     "percent": 200.0
             *                 }
             *             ]
             *         }
             *  One item per event. Each item has stats for each version.
             */
            events_percent_list: []
        }
    },
    /**
     * /versions page's state
     */
    versions: {
        /**
         * List of versions. Single version is {'name':name, 'selection': selection, 'weight': weight, 'url': url}
         */
        list: [],
    },
    /**
     * Backend apps / reverse proxy list
     */
    proxy: {
        /**
         * List of reverse proxies. Ordered by 'prefix DESC' - this is how it's implemented in the backend.
         */
        list: []
    }
});

export function getToken() {
    return state.authorization.token
}

export function setToken(token) {
    state.authorization.token = token
}

export function state_getAnalytics() {
    return state.analytics
}

export function setAnalyticsResults(analytics) {
    state.analytics.totalVisits = analytics.totalVisits
    state.analytics.stats = analytics.stats
}

export function state_setEventsStats(events) {
    state.analytics.events = events
}

export function updateAnalyticsQuery(dateStart, dateEnd) {
    state.analytics.dateStart = dateStart
    state.analytics.dateEnd = dateEnd
}

export function addVersion(name, selection, weight, url) {
    state.versions.list.push({name: name, selection: selection, weight: weight, url: url})
}

export function state_listVersions() {
    return state.versions
}

export function state_deleteVersions() {
    state.versions.list.length = 0
}

export function state_deleteVersion(name) {
    state.versions.list = state.versions.list.filter((e) => e.name !== name)
}

export function state_listProxies() {
    return state.proxy
}


export function state_setProxies(proxies) {
    state.proxy.list = proxies
}
