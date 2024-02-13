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
import {addVersion, deleteVersions, getToken, listVersions} from "../state.js";
import {useToast} from "primevue/usetoast";
import {onMounted, reactive} from "vue";
import Toast from "primevue/toast";
import FileUpload from "primevue/fileupload";
import Card from 'primevue/card';
import Panel from 'primevue/panel';
import Strategy from "./Strategy.vue";
import {http} from "../http.js";
import Button from "primevue/button";

const toast = useToast();
const versions = reactive(listVersions());

const onAdvancedUpload = async (e) => {
  toast.add({severity: 'info', summary: 'Success', detail: 'File Uploaded', life: 3000})
  deleteVersions()
  await loadVersions()
}

async function loadVersions() {
  let versions = listVersions();
  if (versions.list.length === 0) {
    http.get('/list-versions').then(r => {
      for (const version of r.data) {
        addVersion(version.name, version.strategy)
      }
    })
  }
}


async function updateVersions() {
  await http.post(`/update-versions`, versions.list)
}

async function before(event) {
  const token = getToken()
  event.xhr.setRequestHeader("authorization", `Bearer ${token}`)
}

onMounted(() => loadVersions())

</script>

<template>
  <Card>
    <template #title>Installed versions</template>
    <template #content>
      <p v-for="version in versions.list" class="m-0">
        <Panel :header="version.name">
          <Strategy :strategy="version.strategy" :name="version.name"/>
        </Panel>
      </p>
      <Button @click="updateVersions" :disabled="!versions.changed">Update</Button>
    </template>

  </Card>
  <div style="padding: 5px"></div>
  <Card>
    <template #title>Upload a new version</template>
    <template #content>
      <Toast/>
      <FileUpload name="upload[]" url="/upload-version" @upload="onAdvancedUpload($event)" @before-send="before($event)"
                  :multiple="false" accept=".zip">
        <template #empty>
          <p>Drag and drop files to here to upload a new web version</p>
        </template>
      </FileUpload>
    </template>
  </Card>
</template>

<style scoped>

</style>