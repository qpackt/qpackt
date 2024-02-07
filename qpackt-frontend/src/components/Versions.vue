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
  import Strategy from "./Strategy.vue";
  import {http} from "../http.js";
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

  async function deleteVersion(name) {
    await http.delete(`/delete-version/${name}`)
    deleteVersions()
    await loadVersions()
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
  <div class="card">

    <Toast />
    <FileUpload name="demo[]" url="/upload-version" @upload="onAdvancedUpload($event)" @before-send="before($event)" :multiple="false" accept=".zip">
      <template #empty>
        <p>Drag and drop files to here to upload.</p>
      </template>
    </FileUpload>
  </div>
  <div>
    <div v-for="version in versions.list">
      Name: {{version.name}}
      <Strategy :strategy="version.strategy" :name="version.name"/>
      <Button @click="deleteVersion(version.name)">Delete</Button>
    </div>
    <Button @click="updateVersions" :disabled="!versions.changed">Update</Button>
  </div>

</template>

<style scoped>

</style>