<template>
  <v-container
    class="fill-height pa-0"
    fluid
  >
    <div :class="lgAndUp ? 'd-flex flex-row w-100 h-100' : 'd-flex flex-column w-100 h-100'">
      <v-tabs
        v-model="tab"
        :direction="lgAndUp ? 'vertical' : 'horizontal'"
        :align-tabs="lgAndUp ? 'start' : 'center'"
        center-active
        show-arrows
      >
        <v-tab
          slider-color="primary"
          value="content"
        >
          Content
        </v-tab>
        <v-tab
          slider-color="primary"
          value="admins"
        >
          Access
        </v-tab>
        <v-tab
          slider-color="primary"
          value="featured"
        >
          Featured
        </v-tab>
        <v-tab
          slider-color="primary"
          value="subscriptions"
        >
          Subscriptions
        </v-tab>
        <v-tab
          slider-color="primary"
          value="site"
        >
          Site
        </v-tab>
      </v-tabs>
      <v-window
        v-model="tab"
        class="flex-1-0 border-s-sm"
      >
        <v-window-item
          value="content"
        >
          <p>content magnament</p>
        </v-window-item>
        <v-window-item
          value="admins"
        >
          <p>access magnament</p>
        </v-window-item>
        <v-window-item
          value="featured"
        >
          <p>featured magnament</p>
        </v-window-item>
        <v-window-item
          value="subscriptions"
        >
          <p>subcriptions magnament</p>
        </v-window-item>
        <v-window-item
          value="site"
        >
          <p>site magnament</p>
        </v-window-item>
      </v-window>
    </div>
  </v-container>
</template>

<script setup lang="ts">
import {ref} from 'vue';
import {useDisplay} from 'vuetify';

const {lgAndUp} = useDisplay();
const tab = ref(null);
</script>
