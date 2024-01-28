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
import {getAnalytics, setAnalyticsResults, updateAnalyticsQuery} from "../state.js";
import axios from "axios";
import VersionStats from "./VersionStats.vue";

const dateStart = ref(initialPastDate())
const dateEnd = ref(new Date())

const analytics = reactive(getAnalytics())

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
    let request = {
      from_time: dateStart.value.toJSON(),
      to_time: dateEnd.value.toJSON()
    }
    await fetchAnalytics(request)
  }
}

async function fetchAnalytics(request) {
  axios.post('/analytics', request).then(r => {
    setAnalyticsResults({
      totalVisits: r.data.total_visit_count,
      stats: r.data.versions_stats,
    })
  })
}

onMounted(() => loadAnalytics())
</script>

<template>
  <div>
    Analytics template
    <Calendar v-model="dateStart" date-format="yy-mm-dd" showTime hourFormat="24" @date-select="updateAnalyticsQuery(dateStart, dateEnd)"/>
    <Calendar v-model="dateEnd" date-format="yy-mm-dd" showTime hourFormat="24" @date-select="updateAnalyticsQuery(dateStart, dateEnd)"/>
    <div>Total visits: {{analytics.totalVisits}}</div>
    <div v-for="stat in analytics.stats">
      <VersionStats :stats="stat"/>
    </div>
  </div>
</template>

<style scoped>

</style>