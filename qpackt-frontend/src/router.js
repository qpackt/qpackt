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

const Versions = () => import('./components/Versions.vue')
const Analytics = () => import("./components/Analytics.vue")

import { createWebHistory, createRouter } from "vue-router";
import Login from "./components/Login.vue";

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

export default router;