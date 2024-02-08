<script setup>
import Button from "primevue/button";
import {setToken} from "../state.js";
import {http} from "../http.js";

  async function logout() {
    await http.delete('/token')
    setToken('');
  }
</script>

<template>
  <Button @click="logout">Logout</Button>
</template>
