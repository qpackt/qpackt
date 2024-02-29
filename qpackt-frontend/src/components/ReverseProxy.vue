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
<!--along with this program.  If not, see <https://www.gnu.org/licenses/ -->

<script setup>
import {state_listProxies, state_setProxies} from "../state.js";
import {onMounted, reactive, ref} from "vue";
import {http} from "../http.js";
import DataTable from 'primevue/datatable';
import Column from 'primevue/column';
import Panel from "primevue/panel";
import Button from "primevue/button";
import InputText from "primevue/inputtext";

const proxy = reactive(state_listProxies())
const newProxyPrefix = ref('');
const newProxyTarget = ref('');

async function deleteProxy(id) {
  await http.delete(`/proxy/${id}`)
  fetchCurrentProxies()
}

async function addProxy() {
  if (newProxyPrefix.value !== '' && newProxyTarget.value !== '') {
    await http.post('/proxy', {prefix: newProxyPrefix.value, target: newProxyTarget.value})
    newProxyPrefix.value = '';
    newProxyTarget.value = '';
    fetchCurrentProxies()
  }
}

async function loadProxies() {
  if (proxy.list.length === 0) {
    fetchCurrentProxies()
  }
}

function fetchCurrentProxies() {
  http.get("/proxy").then((r) => {
    state_setProxies(r.data)
  })
}

onMounted(() => loadProxies())
</script>

<template>
  <Panel>
    <DataTable :value="proxy.list" table-style="min-width: 50rem" showGridlines stripedRows resizableColumns
               columnResizeMode="fit" style="margin-bottom: 10px">
      <Column field="prefix" header="Endpoint prefix"></Column>
      <Column field="target" header="Target URL"></Column>
      <Column header="Actions">
        <template #body="slotProps">
          <Button @click="deleteProxy(slotProps.data.id)" severity="danger">Delete</Button>
        </template>
      </Column>
    </DataTable>
    <InputGroup>
      <InputGroupAddon>
        <InputText type="text" v-model="newProxyPrefix" placeholder="Endpoint prefix" style="margin-right: 5px"/>
        <InputText type="text" v-model="newProxyTarget" placeholder="Target URL" style="margin-right: 5px"/>
        <Button @click="addProxy()">Add proxy</Button>
      </InputGroupAddon>
    </InputGroup>
  </Panel>
</template>

<style scoped>

</style>