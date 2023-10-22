<!--SPDX-License-Identifier: AGPL-3.0-->

<!--Vaden: Versioned Application Deployment Engine-->
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
  <div class="card">
    <TabMenu v-model:activeIndex="active" :model="items">
      <template #item="{ label, item, props }">
        <router-link v-if="item.route" v-slot="routerProps" :to="item.route" custom>
          <a :href="routerProps.href" v-bind="props.action" @click="($event) => routerProps.navigate($event)" @keydown.enter.space="($event) => routerProps.navigate($event)">
            <span v-bind="props.label">{{ label }}</span>
          </a>
        </router-link>
      </template>
    </TabMenu>
    <div class="container">
      <router-view />
    </div>

  </div>
</template>

<script setup>
import { ref, onMounted, watch } from "vue";
import { useRouter, useRoute } from "vue-router";

import TabMenu from "primevue/tabmenu";

const router = useRouter();
const route = useRoute();

const active = ref(0);
const items = ref([
  {
    label: 'Versions',
    route: '/versions'
  },
  {
    label: 'Analytics',
    route: '/analytics'
  },
]);

onMounted(() => {
  active.value = items.value.findIndex((item) => route.path === router.resolve(item.route).path);
})

watch(
    route,
    () => {
      active.value = items.value.findIndex((item) => route.path === router.resolve(item.route).path);
    },
    { immediate: true }
);
</script>


<style scoped>
.logo {
  height: 6em;
  padding: 1.5em;
  will-change: filter;
  transition: filter 300ms;
}
.logo:hover {
  filter: drop-shadow(0 0 2em #646cffaa);
}
.logo.vue:hover {
  filter: drop-shadow(0 0 2em #42b883aa);
}
</style>
