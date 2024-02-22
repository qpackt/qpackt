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
import {addVersion, state_deleteVersions, getToken, state_listVersions, state_deleteVersion} from "../state.js";
import {onMounted, reactive} from "vue";
import {http} from "../http.js";
import {useToast} from "primevue/usetoast";
import Toast from "primevue/toast";
import FileUpload from "primevue/fileupload";
import Card from 'primevue/card';
import Panel from 'primevue/panel';
import Button from "primevue/button";
import InputGroup from "primevue/inputgroup";
import InputText from "primevue/inputtext";
import RadioButton from "primevue/radiobutton";
import InputGroupAddon from "primevue/inputgroupaddon";
import InputNumber from "primevue/inputnumber";

const toast = useToast();
const versions = reactive(state_listVersions());

const onAdvancedUpload = async (e) => {
  toast.add({severity: 'info', summary: 'Success', detail: 'File Uploaded', life: 3000})
  state_deleteVersions()
  await loadVersions()
}

async function loadVersions() {
  if (versions.list.length === 0) {
    http.get('/versions').then(r => {
      for (const version of r.data) {
        if (version.strategy.UrlParam !== undefined) {
          addVersion(version.name, 'UrlParam', 0, version.strategy.UrlParam)
        } else {
          addVersion(version.name, 'Weight', version.strategy.Weight, '')
        }
      }
    })
  }
}


async function updateVersions() {
  const request = []
  for (const version of versions.list) {
    if (version.selection === 'Weight') {
      request.push({
        name: version.name,
        strategy: {
          Weight: version.weight
        }
      })
    } else {
      request.push({
        name: version.name,
        strategy: {
          UrlParam: version.url
        }
      })
    }
  }
  await http.put(`/versions`, request)
}

async function before(event) {
  const token = getToken()
  event.xhr.setRequestHeader("authorization", `Bearer ${token}`)
}

async function deleteVersion(name) {
  http.delete(`/version/${name}`).then((e) => {
    state_deleteVersion(name)
  })
}

onMounted(() => loadVersions())

</script>

<template>
  <Card>
    <template #title>Installed versions</template>
    <template #content>
      <p v-for="version in versions.list" class="m-0">
        <Panel :header="version.name">
          <InputGroup>
            <InputGroupAddon>
              <RadioButton v-model="version.selection" inputId="Weight" name="selection" value="Weight"/>
              <label for="selection" class="ml-2">&nbsp;Weight&nbsp;</label>
              <InputNumber v-model="version.weight" mode="decimal" :min="0" :max="100"
                           :disabled="version.selection !== 'Weight'"/>
            </InputGroupAddon>
            <InputGroupAddon>
              <RadioButton v-model="version.selection" inputId="UrlParam" name="selection" value="UrlParam"/>
              <label for="selection" class="ml-2">&nbsp;UrlParam&nbsp;</label>
              <InputText type="text" v-model="version.url" :disabled="version.selection !== 'UrlParam'"/>
            </InputGroupAddon>
            <InputGroupAddon>
              <Button @click="deleteVersion(version.name)" severity="danger">Delete</Button>
            </InputGroupAddon>
          </InputGroup>
        </Panel>
      </p>
      <Button @click="updateVersions">Update</Button>
    </template>

  </Card>
  <div style="padding: 5px"></div>
  <Card>
    <template #title>Upload a new version</template>
    <template #content>
      <Toast/>
      <FileUpload name="upload[]" url="/version" @upload="onAdvancedUpload($event)" @before-send="before($event)"
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