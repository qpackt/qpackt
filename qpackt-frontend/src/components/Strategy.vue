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
import {defineProps, onMounted, ref} from 'vue';
import RadioButton from 'primevue/radiobutton';
import InputText from 'primevue/inputtext';
import InputNumber from 'primevue/inputnumber';
import {updateVersion} from "../state.js";


  const props = defineProps(['strategy', 'name']);
  const selection = ref('');
  const urlParam = ref('');
  const weight = ref(0);
  onMounted(() => {
    if (props.strategy.UrlParam !== undefined) {
      selection.value = 'UrlParam'
      urlParam.value = props.strategy.UrlParam
    } else {
      selection.value = 'Weight'
      weight.value = props.strategy.Weight
    }
  })

  function emitCurrent() {
    let value;
    if (selection.value === 'UrlParam') {
      value = {"UrlParam": urlParam.value}
    } else {
      value = {"Weight": weight.value}
    }
    updateVersion(props.name, value)
  }

</script>

<template>
  <div>

    <div class="flex flex-wrap gap-3">

        <RadioButton v-model="selection" inputId="Weight" name="selection" value="Weight" @change="emitCurrent"/>
        <label for="selection" class="ml-2">Weight</label>
        <InputNumber v-model="weight" mode="decimal" :min="0" :max="100" :disabled="selection !== 'Weight'" @focusout="emitCurrent"/>

        <RadioButton v-model="selection" inputId="UrlParam" name="selection" value="UrlParam" @change="emitCurrent"/>
        <label for="selection" class="ml-2">UrlParam</label>
        <InputText type="text" v-model="urlParam" :disabled="selection !== 'UrlParam'" @focusout="emitCurrent"/>

    </div>
  </div>
</template>

<style scoped>

</style>