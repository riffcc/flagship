<template>
  <v-fade-transition>
    <v-sheet
      v-if="gamepadState.connected && showHints"
      color="transparent"
      class="gamepad-hints"
    >
      <div class="hint-container">
        <span class="hint">
          <gamepad-button 
            :type="gamepadState.type" 
            button="a"
          />
          Select
        </span>
        <span class="hint">
          <gamepad-button 
            :type="gamepadState.type" 
            button="b"
          />
          Back
        </span>
        <span class="hint">
          <gamepad-button 
            :type="gamepadState.type" 
            button="x"
          />
          Stop
        </span>
        <span class="hint">
          <gamepad-button 
            :type="gamepadState.type" 
            button="y"
          />
          Filter
        </span>
        <span class="hint">
          <gamepad-button 
            :type="gamepadState.type" 
            button="start"
          />
          Menu
        </span>
        <span class="hint">
          L3/R3 Play/Pause
        </span>
        <span class="hint">
          <gamepad-button 
            :type="gamepadState.type" 
            button="lb"
          />
          /
          <gamepad-button 
            :type="gamepadState.type" 
            button="rb"
          />
          Tabs
        </span>
        <span class="hint">
          <v-icon size="small">gamepad</v-icon>
          Move
        </span>
        <span class="hint">
          <v-icon size="small">cursor-default-outline</v-icon>
          Cursor
        </span>
      </div>
    </v-sheet>
  </v-fade-transition>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';
import type { GamepadState } from '/@/composables/useGamepad';
import GamepadButton from './gamepadButton.vue';

const props = defineProps<{
  gamepadState: GamepadState;
}>();

const showHints = ref(true);
let hasMovedSticks = false;

// Hide hints when sticks are moved
watch([() => props.gamepadState.leftStick, () => props.gamepadState.rightStick], ([leftStick, rightStick]) => {
  if (!hasMovedSticks && (Math.abs(leftStick.x) > 0.2 || Math.abs(leftStick.y) > 0.2 || 
      Math.abs(rightStick.x) > 0.2 || Math.abs(rightStick.y) > 0.2)) {
    hasMovedSticks = true;
    showHints.value = false;
  }
});

// Show hints again when gamepad reconnects
watch(() => props.gamepadState.connected, (connected) => {
  if (connected && !hasMovedSticks) {
    showHints.value = true;
  }
});
</script>

<style scoped>
.gamepad-hints {
  position: fixed;
  bottom: 20px;
  left: 50%;
  transform: translateX(-50%);
  z-index: 1000;
  pointer-events: none;
}

.hint-container {
  display: flex;
  gap: 20px;
  padding: 12px 20px;
  background: rgba(0, 0, 0, 0.9);
  border: 1px solid rgba(138, 43, 226, 0.3);
  border-radius: 8px;
  backdrop-filter: blur(10px);
}

.hint {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 14px;
  color: rgba(255, 255, 255, 0.8);
}
</style>