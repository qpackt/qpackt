<template>
  <div>
    <button type="button" @click="increase">count is {{ count }}</button>
  </div>
</template>


<script>
import { state } from '../state.js'
import { increase } from '../state.js'


export default {
  computed: {
    count() {
      return state.hello_world.count
    }
  },

  methods: {
    increase
  }
}
</script>
