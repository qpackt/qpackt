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

<script setup>
  import { increase } from "../state.js";
  import { useToast } from "primevue/usetoast";
  const toast = useToast();

  const onAdvancedUpload = (e) => {
    toast.add({ severity: 'info', summary: 'Success', detail: 'File Uploaded', life: 3000 });
  };
</script>

<template>
  Versions template
  <button type="button" @click="increase">Click me</button>
  <div class="card">
    <Toast />
    <FileUpload name="demo[]" url="/upload-version" @upload="onAdvancedUpload($event)" :multiple="false" accept=".zip">
      <template #empty>
        <p>Drag and drop files to here to upload.</p>
      </template>
    </FileUpload>
  </div>
</template>

<style scoped>

</style>