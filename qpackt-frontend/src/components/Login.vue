<!--SPDX-License-Identifier: AGPL-3.0-->

<!--qpackt: Web & Analytics Server-->
<!--Copyright (C) 2023 Łukasz Wojtów-->

<!--This program is free software: you can redistribute it and/or modify-->
<!--it under the terms of the GNU Affero General Public License as-->
<!--published by the Free Software Foundation, either version 3 of the-->
<!--License.-->

<!--This program is distributed in the hope that it will be useful,-->
<!--but WITHOUT ANY WARRANTY; without even the implied warranty of-->
<!--MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the-->
<!--GNU Affero General Public License for more details.-->

<!--You should have received a copy of the GNU Affero General Public License-->
<!--along with this program.  If not, see <https://www.gnu.org/licenses/>.-->


<script setup>
import {setToken} from "../state.js";
import Button from 'primevue/button';
import Password from 'primevue/password';
import {ref} from "vue";
import {http} from "../http.js";

function haveToken(token) {
  setToken(token.token);
}

async function fetchToken() {
  http.post('/token', {password: password.value}).then(r => haveToken(r.data))
}

const password = ref('');
</script>

<template>
  <div>
    Qpackt Login
    <div style="padding: 10px">
      <Password v-model="password" :feedback="false"/>
    </div>
    <div>
      <Button label="Login" @click="fetchToken"/>
    </div>

  </div>
</template>
