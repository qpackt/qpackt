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
import {setAnalyticsResults, state_getAnalytics, state_setEventsStats, updateAnalyticsQuery} from "../state.js";
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

function setEventsResults(data) {
  state_setEventsStats(data)
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
  http.get(`/events/stats?from_time=${request.from_time}&to_time=${request.to_time}`).then(r => {
    setEventsResults(r.data)
  })
}

async function downloadEvents() {
  let request = {
    from_time: dateStart.value.toJSON(),
    to_time: dateEnd.value.toJSON()
  }
  const csvUrl = `/events/csv?from_time=${request.from_time}&to_time=${request.to_time}`
  const filename = 'events.csv'
  await http.downloadCsv(csvUrl, filename)
}

onMounted(() => loadAnalytics())
</script>

<template>
  <Panel>
    <InputGroup>
      <InputGroupAddon>
        <label class="ml-2" for="selection">From&nbsp;</label>
        <Calendar v-model="dateStart" date-format="yy-mm-dd" hourFormat="24" showTime
                  @date-select="updateAnalyticsQuery(dateStart, dateEnd)"/>
      </InputGroupAddon>
      <InputGroupAddon>
        <label class="ml-2" for="selection">&nbsp;to&nbsp;</label>
        <Calendar v-model="dateEnd" date-format="yy-mm-dd" hourFormat="24" showTime
                  @date-select="updateAnalyticsQuery(dateStart, dateEnd)"/>
      </InputGroupAddon>
      <InputGroupAddon>
        <Button @click="fetchAnalytics">Get</Button>
      </InputGroupAddon>
    </InputGroup>
  </Panel>
  <div style="padding: 10px"></div>
  <Panel>
    Total visits: {{ analytics.totalVisits }}
    <DataTable :value="analytics.stats" showGridlines stripedRows table-style="min-width: 50rem">
      <Column field="name" header="Name"></Column>
      <Column field="visit_count" header="Visit count"></Column>
      <Column field="average_requests" header="Av. requests"></Column>
      <Column field="average_duration" header="Av. duration (sec)"></Column>
      <Column field="bounce_rate" header="Bounce rate"></Column>
    </DataTable>
  </Panel>
  <div style="padding: 10px"></div>
  <Button @click="downloadEvents">Download events</Button>
  <div v-for="item in analytics.events.events_percent_list">
    <div style="padding: 10px"></div>
    <Panel>
      {{ item.event }}
      <DataTable :value="item.percents" showGridlines stripedRows table-style="min-width: 50rem">
        <Column field="version" header="Version"></Column>
        <Column field="percent" header="Percent"></Column>
      </DataTable>
    </Panel>
  </div>

</template>
