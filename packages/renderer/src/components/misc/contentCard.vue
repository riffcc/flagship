<template>
  <v-hover
    v-slot="{props: hoveringProps, isHovering}"
    open-delay="150"
    close-delay="150"
  >
    <v-sheet
      v-bind="hoveringProps"
      :class="cursorPointer ? 'cursor-pointer mx-auto' : 'mx-auto'"
      color="transparent"
      :height="height"
      :width="width"
      :style="showDefederation ? `border: 1px solid ${lensColorHash(item)};` : ''"
      @click="onClick"
    >
      <template v-if="overlapping">
        <v-img
          :src="backgroundImage"
          width="100%"
          cover
          aspect-ratio="1"
          :gradient="backgroundGradient"
        >
          <p class="ml-4 mt-2 text-subtitle-1">
            {{ title }}
          </p>
          <p class="ml-4 text-subtitle-2">
            {{ subtitle }}
          </p>
          <slot
            v-if="isHovering"
            name="hovering"
          ></slot>
          <slot
            name="actions"
          ></slot>
        </v-img>
      </template>
      <template v-else>
        <v-img
          :src="backgroundImage"
          width="100%"
          cover
          aspect-ratio="1"
        >
          <slot
            v-if="isHovering"
            name="hovering"
          ></slot>
        </v-img>
        <p class="text-caption text-sm-subtitle-1 text-center mt-1">
          {{ title }}
        </p>
        <p class="text-caption text-sm-subtitle-1 text-center text-medium-emphasis">
          {{ subtitle }}
        </p>
      </template>
    </v-sheet>
  </v-hover>
</template>

<script setup lang="ts">
import { useShowDefederation } from '/@/composables/showDefed';
import { lensColorHash } from '/@/utils';


const {showDefederation} = useShowDefederation();


defineProps<{
  backgroundGradient?: string;
  backgroundImage: string;
  cursorPointer?: boolean;
  height?: string | number;
  overlapping?: boolean;
  subtitle: string;
  title: string;
  width?: string | number;
  onClick?: () => void;
}>();
</script>
