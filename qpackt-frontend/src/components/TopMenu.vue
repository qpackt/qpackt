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
import {onMounted, ref, watch} from "vue";
import {useRoute, useRouter} from "vue-router";
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
  {
    label: 'Reverse Proxies',
    route: '/proxies'
  },
]);


onMounted(() => {
  active.value = items.value.findIndex((item) => route.path === router.resolve(item.route).path);
  if (active.value === -1) {
    active.value = 0
  }
})

watch(
    route,
    () => {
      active.value = items.value.findIndex((item) => route.path === router.resolve(item.route).path);
      if (active.value === -1) {
        active.value = 0
      }
    },
    {immediate: true}
);
</script>

<template>
  <div>
    <TabMenu v-model:activeIndex="active" :model="items">
      <template #item="{ label, item, props }">
        <router-link v-if="item.route" v-slot="routerProps" :to="item.route" custom>
          <a :href="routerProps.href" v-bind="props.action" @click="($event) => routerProps.navigate($event)"
             @keydown.enter.space="($event) => routerProps.navigate($event)">
            <span v-bind="props.label">{{ label }}</span>
          </a>
        </router-link>
      </template>
    </TabMenu>
  </div>
</template>

<style scoped>

</style>