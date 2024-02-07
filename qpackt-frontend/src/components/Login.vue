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

  function loggedIn(r) {
    // this.loading = false;
    // let role = r.data.role;
    // let token = r.data.token.value;
    // let user_id = r.data.user_id;
    // this.$store.commit('save_token', token);
    // this.$store.commit('save_role', role);
    // this.$store.commit('save_user_id', user_id);
    // if (this.remember_password) {
    //   localStorage.setItem('login', this.login.login.trim());
    //   localStorage.setItem('password', this.login.password.trim());
    // }
    // if (role === 'Admin') {
    //   this.$router.push({name:'admin'})
    // } else {
    //   if (this.$route.query.next !== undefined) {
    //     this.$router.push({path: this.$route.query.next})
    //   } else {
    //     this.$router.push({name:'jobs'})
    //   }
    // }
  }

  function haveToken(token) {
    setToken(token.token);
  }

  function loginError() {
    console.log('error')
  }
  async function fetchToken() {
    http.post('/token', {password: password.value}).then(r => haveToken(r.data)).catch(r => loginError())
  }

  const password = ref('');
</script>

<template>
  <div>
    <div class="card flex justify-content-center">
      <Password v-model="password" :feedback="false"/>
    </div>
    <Button label="Login" @click="fetchToken"/>
  </div>
</template>

<style scoped>

</style>