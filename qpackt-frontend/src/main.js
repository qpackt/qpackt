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

import './style.css'
import "primevue/resources/themes/md-light-indigo/theme.css";

import { createApp } from 'vue'
import PrimeVue from 'primevue/config';
import router from './router'
import ToastService from "primevue/toastservice";

import App from './App.vue'

const app = createApp(App)

app.use(router)
app.use(ToastService)
app.use(PrimeVue, { ripple: true })

app.mount('#app')
