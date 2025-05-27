<template>
  <div>
    <v-card class="pa-4">
      <v-card-title>Register to upload content</v-card-title>
      <v-card-text>
        <p v-if="!isRegistered">Complete a simple proof of work to register and get upload access.</p>
        <p v-else>You are now registered and can upload content!</p>
        
        <div v-if="!isWorking && !isRegistered" class="mt-4">
          <v-btn 
            color="primary" 
            block
            @click="startRegistration"
          >
            Start Registration
          </v-btn>
        </div>
        
        <div v-if="isWorking" class="mt-4">
          <v-progress-circular
            indeterminate
            color="primary"
            class="mb-4"
          ></v-progress-circular>
          <p>Performing proof of work... {{ powProgress }}%</p>
        </div>
        
        <div v-if="isRegistered" class="mt-4">
          <v-alert
            type="success"
            variant="tonal"
          >
            Registration successful! Your ID: {{ userId }}
          </v-alert>
          <v-btn
            color="primary"
            block
            class="mt-4"
            :to="{ name: 'upload' }"
          >
            Start Uploading
          </v-btn>
        </div>
      </v-card-text>
    </v-card>
  </div>
</template>

<script setup lang="ts">
import { ref, inject, onMounted } from 'vue';
import type { Peerbit } from 'peerbit';

// States
const isWorking = ref(false);
const isRegistered = ref(false);
const powProgress = ref(0);
const userId = ref('');

// Get Peerbit instance
const peerbit = inject<Peerbit>('peerbit');

// Check if already registered
onMounted(async () => {
  if (peerbit?.libp2p?.peerId) {
    userId.value = peerbit.libp2p.peerId.toString();
    
    // In a real implementation, you'd check against a database/store
    // For now, we'll simulate by checking localStorage
    const isUserRegistered = localStorage.getItem('user_registered');
    if (isUserRegistered === 'true') {
      isRegistered.value = true;
    }
  }
});

// Simple proof of work simulation
const startRegistration = async () => {
  isWorking.value = true;
  powProgress.value = 0;
  
  // Simulate proof of work with a timer
  const totalSteps = 20;
  const interval = setInterval(() => {
    powProgress.value += 100 / totalSteps;
    
    if (powProgress.value >= 100) {
      clearInterval(interval);
      completeRegistration();
    }
  }, 150); // Complete in about 3 seconds
  
  // In a real implementation, you'd do actual work here
  // For example, find a hash with specific properties
};

const completeRegistration = async () => {
  // In a real implementation, you'd register this user in a database
  // For now, we'll just set a flag in localStorage
  localStorage.setItem('user_registered', 'true');
  
  // Update state
  isWorking.value = false;
  isRegistered.value = true;
};
</script> 