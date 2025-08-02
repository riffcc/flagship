<template>
  <span class="gamepad-button" :class="type">
    {{ buttonDisplay }}
  </span>
</template>

<script setup lang="ts">
import { computed } from 'vue';

const props = defineProps<{
  type: 'xbox' | 'playstation' | 'switch' | 'generic';
  button: string;
}>();

const buttonMappings = {
  xbox: {
    a: 'A',
    b: 'B',
    x: 'X',
    y: 'Y',
    lb: 'LB',
    rb: 'RB',
    lt: 'LT',
    rt: 'RT',
    start: 'Menu',
    select: 'View',
  },
  playstation: {
    a: '✕',
    b: '○',
    x: '□',
    y: '△',
    lb: 'L1',
    rb: 'R1',
    lt: 'L2',
    rt: 'R2',
    start: 'Options',
    select: 'Share',
  },
  switch: {
    a: 'B',
    b: 'A',
    x: 'Y',
    y: 'X',
    lb: 'L',
    rb: 'R',
    lt: 'ZL',
    rt: 'ZR',
    start: '+',
    select: '-',
  },
  generic: {
    a: '1',
    b: '2',
    x: '3',
    y: '4',
    lb: 'L1',
    rb: 'R1',
    lt: 'L2',
    rt: 'R2',
    start: 'Start',
    select: 'Select',
  },
};

const buttonDisplay = computed(() => {
  return buttonMappings[props.type][props.button as keyof typeof buttonMappings.xbox] || props.button;
});
</script>

<style scoped>
.gamepad-button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 24px;
  height: 24px;
  padding: 0 6px;
  background: rgba(138, 43, 226, 0.2);
  border: 1px solid rgba(138, 43, 226, 0.6);
  border-radius: 4px;
  font-size: 12px;
  font-weight: bold;
  color: #fff;
}

.gamepad-button.playstation {
  font-family: Arial, sans-serif;
}

.gamepad-button.xbox {
  border-radius: 50%;
  min-width: 24px;
  padding: 0;
}

.gamepad-button.switch {
  border-radius: 50%;
  min-width: 24px;
  padding: 0;
}
</style>