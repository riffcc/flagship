<template>
  <div v-if="message" class="toast">
    {{ message }}
  </div>
</template>

<script setup>
defineProps({
  message: String,
})
</script>

<style scoped>
.toast {
  position: fixed;
  bottom: 2rem;
  right: 2rem;
  background: rgba(0, 255, 136, 0.95);
  color: #0a0a0a;
  padding: 1rem 1.5rem;
  border-radius: 8px;
  box-shadow: 0 4px 12px rgba(0, 255, 136, 0.3);
  font-weight: bold;
  z-index: 1000;
  animation: slideIn 0.3s ease-out;
}

@keyframes slideIn {
  from {
    transform: translateY(100px);
    opacity: 0;
  }
  to {
    transform: translateY(0);
    opacity: 1;
  }
}
</style>
