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
 * Also, state should not be accessed directly from components but modification functions should be called.
 * See examples in components for usage.
 *
 */
const state = reactive({
    /**
     * Each component has its own 'substate' to keep things organized.
     */
  hello_world: {
    count: 0,
  },
});

export async function increase() {
    state.hello_world.count += 1
}

export function get_count() {
    return state.hello_world.count
}