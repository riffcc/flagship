<template>
  <v-container
    fill-height
    class="d-flex flex-column ga-4 my-10"
  >
    <h1>About</h1>
    <p>{{ aboutRiffCC.about.motto }}</p>
    <p>{{ aboutRiffCC.about.description }}</p>

    <v-list bg-color="transparent">
      <v-list-title>
        <h2>Team</h2>
      </v-list-title>
      <v-list-item
        v-for="(member, index) in aboutRiffCC.team"
        :key="index"
        :title="member.name"
        :subtitle="`${member.role} (${member.affiliation})`"
      >
      </v-list-item>
    </v-list>

    <p>{{ aboutRiffCC.acknowledgement }}</p>

    <h2>Structure</h2>
    <h3>Riff Labs</h3>
    <p>{{ aboutRiffCC.structure.riffLabs.description }}</p>
    <p>{{ aboutRiffCC.structure.riffLabs.mission }}</p>

    <h3>Riff.CC</h3>
    <p>{{ aboutRiffCC.structure.riffCC.description }}</p>
    <p>{{ aboutRiffCC.structure.riffCC.funding }}</p>
  </v-container>
</template>

<script setup lang="ts">
import {aboutRiffCC} from '../constants/about';
</script>
