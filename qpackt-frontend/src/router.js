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

import {getToken} from "./state.js";

const Versions = () => import('./components/Versions.vue')
const Analytics = () => import("./components/Analytics.vue")

import { createWebHistory, createRouter } from "vue-router";
import * as state from "./state.js";
import Login from "./components/Login.vue";
import {http} from "./http.js";

const routes = [
    {
        path: "/",
        name: "root",
        component: Versions,
        meta: {
            requiresAuth: true,
        }
    },
    {
        path: "/analytics",
        name: "analytics",
        component: Analytics,
        meta: {
            requiresAuth: true,
        }
    },
    {
        path: "/login",
        name: "login",
        component: Login,
        meta: {
            requiresAuth: false,
        }
    },
    {
        path: "/versions",
        name: "versions",
        component: Versions,
        meta: {
            requiresAuth: true,
        }
    },
]

const router = createRouter({
    routes,
    history: createWebHistory()
});

router.beforeEach((to, from, next) => {
    let has_auth = state.getToken !== 0;
    if (to.meta.requiresAuth) {
        if (has_auth) {
            next();
        } else {
            http.try_login().then(response => {
                if (response) {
                    next();
                } else {
                    next({name: 'login', query: {next: to.path}})
                }
            }).catch(() => {
                next({name: 'login', query: {next: to.path}})
            });
        }
    } else {
        if (!has_auth) {
            http.try_login().finally( () => next())
        } else {
            next();
        }

    }
})

export default router;