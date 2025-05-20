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
        <v-tab
          slider-color="primary"
          value="categories"
        >
          Categories
        </v-tab>
      </v-tabs>
      <v-window
        v-model="tab"
        class="flex-1-0 border-s-sm"
      >
        <v-window-item
          value="content"
        >
          <content-management @feature-release="handleFeatureReleaseRequest"></content-management>
        </v-window-item>
        <v-window-item
          value="admins"
        >
          <access-management></access-management>
        </v-window-item>
        <v-window-item
          value="featured"
        >
          <featured-management
            :initial-feature-data="initialFeatureData"
            @initial-data-consumed="clearInitialFeatureData"
          ></featured-management>
        </v-window-item>
        <v-window-item
          value="subscriptions"
        >
          <subscription-management></subscription-management>
        </v-window-item>
        <v-window-item
          value="site"
        >
          <site-management></site-management>
        </v-window-item>
        <v-window-item
          value="categories"
        >
          <categories-management></categories-management>
        </v-window-item>
      </v-window>
    </div>
  </v-container>
</template>

<script setup lang="ts">
import {ref, type Ref} from 'vue';
import {useDisplay} from 'vuetify';
import contentManagement from '/@/components/admin/contentManagement.vue';
import accessManagement from '/@/components/admin/accessManagement.vue';
import featuredManagement from '/@/components/admin/featuredManagement.vue';
import subscriptionManagement from '/@/components/admin/subscriptionManagement.vue';
import siteManagement from '/@/components/admin/siteManagement.vue';
import categoriesManagement from '/@/components/admin/categoriesManagement.vue';
import type { PartialFeaturedReleaseItem } from '/@//types';

const {lgAndUp} = useDisplay();
const tab = ref('content');

const initialFeatureData: Ref<PartialFeaturedReleaseItem | null> = ref(null);
const handleFeatureReleaseRequest = async (releaseId: string) => {
  const now = new Date();
  const tomorrow = new Date(now);
  tomorrow.setMonth(now.getMonth() + 1);

  initialFeatureData.value = {
    releaseId: releaseId,
    startTime: now.toISOString().substring(0, 16),
    endTime: tomorrow.toISOString().substring(0, 16),
    promoted: true,
  };
  tab.value = 'featured';
};

// NEW: Handler to clear the initial data once consumed by featuredManagement
const clearInitialFeatureData = () => {
  initialFeatureData.value = null;
};
</script>
