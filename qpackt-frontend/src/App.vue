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

<template>
  <div v-if="getToken() !== ''">
    <div class="right">
      <Logout/>
    </div>
    <div class="container">
      <TopMenu/>
    </div>
    <router-view/>
  </div>
  <div v-else>
    <div class="container">
      <Login/>
    </div>
  </div>
</template>

<script setup>
import {getToken} from "./state.js";
import Login from "./components/Login.vue";
import TopMenu from "./components/TopMenu.vue";
import Logout from "./components/Logout.vue";
</script>

<style scoped>
body {
  margin: 0;
  padding: 0;
  height: 100vh;
  display: flex;
  justify-content: center;
  align-items: center;
}

.container {
  text-align: center;
  padding: 10px;
}

.right {
  text-align: right;
}

</style>