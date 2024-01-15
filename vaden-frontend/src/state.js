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

import { reactive } from 'vue';

/**
 * This is application state. Implementation provides the same functionality as vuex or redux but with slightly
 * better api. Components are not supposed to have own data but use getters instead.
 * Also, state should not be accessed directly from components. Instead, modification functions should be called.
 * Each component has its own 'substate' to keep things organized.
 * See examples in components for usage.
 *
 */
const state = reactive({
    /** Analytics page state
     *
     */
    analytics: {
        dateStart: {},
        dateEnd: {},
        totalVisits: 0,
        stats: [],
    },
    /**
     * /versions page's state
     */
    versions: {
        /**
         * Indicates whether there were some changes made. Used to enable "Update" button
         */
        changed: false,
        /**
         * List of versions. Single version is {'name':name, 'strategy':strategy}
         */
        list: [],
    }
});

export function getAnalytics() {
    return state.analytics
}

export function setAnalyticsResults(analytics) {
    state.analytics.totalVisits = analytics.totalVisits
    state.analytics.stats = analytics.stats
}

export function updateAnalyticsQuery(dateStart, dateEnd) {
    state.analytics.dateStart = dateStart
    state.analytics.dateEnd = dateEnd
}

export function addVersion(name, value) {
    state.versions.list.push({name: name, strategy: value})
}

export function updateVersion(name, value) {
    for (const version of state.versions.list) {
        if (version.name === name) {
            version.strategy = value
            break
        }
    }
    state.versions.changed = true
}

export function listVersions() {
    return state.versions
}

export function deleteVersions() {
    state.versions.changed = false
    state.versions.list.length = 0
}
