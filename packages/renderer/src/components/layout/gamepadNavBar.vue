<template>
  <nav class="gamepad-nav-bar">
    <div class="nav-container">
      <router-link to="/" class="logo-link">
        <img src="/logo.svg" alt="Riff.CC" class="logo" />
      </router-link>
      <router-link
        v-for="(tab, index) in tabs"
        :key="tab.path"
        :to="tab.path"
        class="nav-tab"
        :class="{ active: currentPath === tab.path }"
        :data-navigable="true"
        :tabindex="index"
      >
        <span class="tab-text">{{ tab.name }}</span>
      </router-link>
      <router-link to="/account" class="profile-link" :data-navigable="true">
        <v-icon size="32" color="white">account</v-icon>
      </router-link>
    </div>
  </nav>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useRoute } from 'vue-router';

const route = useRoute();

const tabs = [
  { name: 'Home', path: '/' },
  { name: 'Music', path: '/featured/music' },
  { name: 'Movies', path: '/featured/movies' },
  { name: 'TV Shows', path: '/featured/tv-shows' },
];

const currentPath = computed(() => route.path);
</script>

<style scoped>
.gamepad-nav-bar {
  position: fixed;
  top: 1px;
  left: 0;
  right: 0;
  z-index: 999;
  background: rgba(0, 0, 0, 0.95);
  backdrop-filter: blur(10px);
  border-bottom: 1px solid rgba(138, 43, 226, 0.2);
}

.nav-container {
  display: flex;
  justify-content: center;
  align-items: center;
  height: 55px;
  gap: 40px;
  padding: 0 20px;
  position: relative;
}

.logo-link {
  position: absolute;
  left: 15%;
  top: 50%;
  transform: translateY(-50%);
  display: flex;
  align-items: center;
  height: 48px;
  outline: none;
}

.logo {
  height: 48px;
  width: auto;
  filter: brightness(0.9);
  transition: filter 0.3s ease;
}

.logo-link:hover .logo {
  filter: brightness(1.1);
}

.logo-link:focus {
  box-shadow: 0 0 0 2px rgba(138, 43, 226, 0.6);
  border-radius: 4px;
}

.profile-link {
  position: absolute;
  right: 15%;
  top: 50%;
  transform: translateY(-50%);
  display: flex;
  align-items: center;
  justify-content: center;
  width: 40px;
  height: 40px;
  color: rgba(255, 255, 255, 0.9);
  transition: all 0.3s ease;
  border-radius: 50%;
  outline: none;
  background: rgba(255, 255, 255, 0.05);
}

.profile-link:hover {
  color: rgba(255, 255, 255, 0.95);
  background: rgba(255, 255, 255, 0.1);
}

.profile-link:focus {
  box-shadow: 0 0 0 2px rgba(138, 43, 226, 0.6);
  background: rgba(138, 43, 226, 0.1);
}

.nav-tab {
  position: relative;
  color: rgba(255, 255, 255, 0.7);
  text-decoration: none;
  font-size: 16px;
  font-weight: 500;
  padding: 12px 16px 4px 16px;
  transition: all 0.3s ease;
  outline: none;
}

.nav-tab:hover {
  color: rgba(255, 255, 255, 0.95);
}

.nav-tab.active {
  color: #fff;
}

.nav-tab::after {
  content: '';
  position: absolute;
  bottom: -13px;
  left: 0;
  right: 0;
  height: 2px;
  background: #8a2be2;
  box-shadow: 0 0 10px rgba(138, 43, 226, 0.8);
  opacity: 0;
  transition: opacity 0.3s ease;
}

.nav-tab.active::after {
  opacity: 1;
}

.nav-tab:focus::after {
  opacity: 0.5;
}

.nav-tab:focus {
  /* No focus box - just rely on the underline */
  outline: none;
}

.tab-text {
  position: relative;
  z-index: 1;
}
</style>