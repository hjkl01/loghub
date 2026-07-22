<template>
  <div class="swagger-container">
    <iframe
      :src="swaggerUrl"
      frameborder="0"
      class="swagger-iframe"
    />
  </div>
</template>

<script setup lang="ts">
const swaggerUrl = `${window.location.origin}/swagger-ui/`
</script>

<style scoped>
.swagger-container {
  height: calc(100vh - 120px);
  width: 100%;
}
.swagger-iframe {
  width: 100%;
  height: 100%;
  border: none;
}
</style>
