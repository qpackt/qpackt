
<template>
  <div>
    <button type="button" @click="inc">count is {{ count }}</button>
  </div>
</template>


<script>
import { ref } from 'vue'
import { state } from '../state.js'
import { increase } from '../state.js'


export default {
  computed: {
    count() {
      return state.hello_world.count
    }
  },

  methods: {
    inc() {
      increase()
    }
  }
}
</script>
