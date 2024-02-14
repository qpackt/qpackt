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
import {onMounted, reactive, ref} from "vue";
import Calendar from 'primevue/calendar';
import InputGroup from 'primevue/inputgroup';
import InputGroupAddon from 'primevue/inputgroupaddon';
import Button from "primevue/button";
import DataTable from 'primevue/datatable';
import Column from 'primevue/column';
import Panel from 'primevue/panel';
import {state_getAnalytics, setAnalyticsResults, updateAnalyticsQuery} from "../state.js";
import {http} from "../http.js";

const dateStart = ref(initialPastDate())
const dateEnd = ref(new Date())
const analytics = reactive(state_getAnalytics())

function initialPastDate() {
  let date = new Date()
  date.setDate(date.getDate() - 7)
  return date
}

async function loadAnalytics() {
  if (analytics.dateStart instanceof Date) {
    dateStart.value = analytics.dateStart
    dateEnd.value = analytics.dateEnd
  } else {
    await fetchAnalytics()
  }
}

async function fetchAnalytics() {
  let request = {
    from_time: dateStart.value.toJSON(),
    to_time: dateEnd.value.toJSON()
  }
  http.post('/analytics', request).then(r => {
    setAnalyticsResults({
      totalVisits: r.data.total_visit_count,
      stats: r.data.versions_stats,
    })
  })
}

onMounted(() => loadAnalytics())
</script>

<template>
  <Panel>
    <InputGroup>
      <InputGroupAddon>
        <label for="selection" class="ml-2">From&nbsp;</label>
        <Calendar v-model="dateStart" date-format="yy-mm-dd" showTime hourFormat="24"
                  @date-select="updateAnalyticsQuery(dateStart, dateEnd)"/>
      </InputGroupAddon>
      <InputGroupAddon>
        <label for="selection" class="ml-2">&nbsp;to&nbsp;</label>
        <Calendar v-model="dateEnd" date-format="yy-mm-dd" showTime hourFormat="24"
                  @date-select="updateAnalyticsQuery(dateStart, dateEnd)"/>
      </InputGroupAddon>
      <InputGroupAddon>
        <Button @click="fetchAnalytics">Get</Button>
      </InputGroupAddon>
    </InputGroup>
  </Panel>
  <div style="padding: 10px"></div>
  <Panel>
    Total visits: {{analytics.totalVisits}}
    <DataTable :value="analytics.stats" table-style="min-width: 50rem" showGridlines stripedRows>
      <Column field="name" header="Name"></Column>
      <Column field="visit_count" header="Visit count"></Column>
      <Column field="average_requests" header="Av. requests"></Column>
      <Column field="average_duration" header="Av. duration (sec)"></Column>
      <Column field="bounce_rate" header="Bounce rate"></Column>
    </DataTable>
  </Panel>
</template>
