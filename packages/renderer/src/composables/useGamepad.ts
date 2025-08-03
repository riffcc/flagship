import { ref, onMounted, onUnmounted, computed } from 'vue';
import { useInputMethod } from './useInputMethod';

export interface GamepadState {
  connected: boolean;
  type: 'xbox' | 'playstation' | 'switch' | 'generic';
  leftStick: { x: number; y: number };
  rightStick: { x: number; y: number };
  buttons: {
    a: boolean;
    b: boolean;
    x: boolean;
    y: boolean;
    lb: boolean;
    rb: boolean;
    lt: number;
    rt: number;
    start: boolean;
    select: boolean;
    leftStickButton: boolean;
    rightStickButton: boolean;
    up: boolean;
    down: boolean;
    left: boolean;
    right: boolean;
  };
}

const DEAD_ZONE = 0.15;
const TRIGGER_DEAD_ZONE = 0.1; // Dead zone for analog triggers
const SCROLL_SPEED_MIN = 1;
const SCROLL_SPEED_MAX = 20;

export function useGamepad() {
  const { setInputMethod } = useInputMethod();
  
  const gamepadState = ref<GamepadState>({
    connected: false,
    type: 'generic',
    leftStick: { x: 0, y: 0 },
    rightStick: { x: 0, y: 0 },
    buttons: {
      a: false,
      b: false,
      x: false,
      y: false,
      lb: false,
      rb: false,
      lt: 0,
      rt: 0,
      start: false,
      select: false,
      leftStickButton: false,
      rightStickButton: false,
      up: false,
      down: false,
      left: false,
      right: false,
    },
  });

  const gamepadIndex = ref<number | null>(null);
  const previousButtonStates = new Map<string, boolean>();
  const buttonCallbacks = new Map<string, () => void>();
  let hasRecentInput = false;

  // Detect gamepad type from ID
  function detectGamepadType(id: string): 'xbox' | 'playstation' | 'switch' | 'generic' {
    const lowerId = id.toLowerCase();
    if (lowerId.includes('xbox') || lowerId.includes('microsoft')) return 'xbox';
    if (lowerId.includes('playstation') || lowerId.includes('dualshock') || lowerId.includes('dualsense')) return 'playstation';
    if (lowerId.includes('switch') || lowerId.includes('nintendo')) return 'switch';
    return 'generic';
  }

  // Apply dead zone to stick values
  function applyDeadZone(value: number): number {
    if (Math.abs(value) < DEAD_ZONE) return 0;
    const sign = value > 0 ? 1 : -1;
    return sign * ((Math.abs(value) - DEAD_ZONE) / (1 - DEAD_ZONE));
  }

  // Calculate scroll speed based on stick pressure
  function calculateScrollSpeed(value: number): number {
    const absValue = Math.abs(value);
    return SCROLL_SPEED_MIN + (SCROLL_SPEED_MAX - SCROLL_SPEED_MIN) * absValue;
  }

  // Register button press callback
  function onButtonPress(button: string, callback: () => void) {
    buttonCallbacks.set(button, callback);
  }

  // Update gamepad state
  function updateGamepad() {
    if (gamepadIndex.value === null) return;

    const gamepads = navigator.getGamepads();
    const gamepad = gamepads[gamepadIndex.value];
    
    if (!gamepad) {
      gamepadState.value.connected = false;
      return;
    }
    
    // Debug: Log raw input - commented out to reduce console spam
    // if (Math.random() < 0.016) {
    //   console.log('Gamepad state:', {
    //     axes: gamepad.axes.slice(0, 4),
    //     buttons: gamepad.buttons.slice(0, 16).map(b => b?.pressed)
    //   });
    // }

    // Update stick positions with dead zone
    const leftX = applyDeadZone(gamepad.axes[0]);
    const leftY = applyDeadZone(gamepad.axes[1]);
    const rightX = applyDeadZone(gamepad.axes[2]);
    const rightY = applyDeadZone(gamepad.axes[3]);
    
    gamepadState.value.leftStick = { x: leftX, y: leftY };
    gamepadState.value.rightStick = { x: rightX, y: rightY };
    
    // Check if there's any gamepad input
    const hasStickInput = Math.abs(leftX) > 0 || Math.abs(leftY) > 0 || 
                         Math.abs(rightX) > 0 || Math.abs(rightY) > 0;

    // Map buttons (standard gamepad mapping)
    const buttonMappings = {
      a: gamepad.buttons[0],
      b: gamepad.buttons[1],
      x: gamepad.buttons[2],
      y: gamepad.buttons[3],
      lb: gamepad.buttons[4],
      rb: gamepad.buttons[5],
      lt: gamepad.buttons[6],
      rt: gamepad.buttons[7],
      select: gamepad.buttons[8],
      start: gamepad.buttons[9],
      leftStickButton: gamepad.buttons[10],
      rightStickButton: gamepad.buttons[11],
      up: gamepad.buttons[12],
      down: gamepad.buttons[13],
      left: gamepad.buttons[14],
      right: gamepad.buttons[15],
    };

    // Check if any button is pressed
    let hasButtonInput = false;

    // Update button states and trigger callbacks
    Object.entries(buttonMappings).forEach(([key, button]) => {
      if (!button) return; // Skip if button doesn't exist
      
      const pressed = button.pressed;
      const value = button.value;
      
      if (key === 'lt' || key === 'rt') {
        // Apply dead zone to triggers
        const triggerValue = value > TRIGGER_DEAD_ZONE ? value : 0;
        (gamepadState.value.buttons as any)[key] = triggerValue;
        if (triggerValue > 0) hasButtonInput = true;
      } else {
        const wasPressed = previousButtonStates.get(key) || false;
        (gamepadState.value.buttons as any)[key] = pressed;
        if (pressed) hasButtonInput = true;
        
        // Trigger callback on button press (not release)
        if (pressed && !wasPressed && buttonCallbacks.has(key)) {
          buttonCallbacks.get(key)!();
        }
        
        previousButtonStates.set(key, pressed);
      }
    });
    
    // Set input method to gamepad if there's any input
    if (hasStickInput || hasButtonInput) {
      if (!hasRecentInput) {
        setInputMethod('gamepad');
        hasRecentInput = true;
      }
    } else {
      hasRecentInput = false;
    }
  }

  // Computed values for easier access
  const scrollSpeed = computed(() => ({
    x: calculateScrollSpeed(gamepadState.value.leftStick.x),
    y: calculateScrollSpeed(gamepadState.value.leftStick.y),
  }));

  const isMoving = computed(() => 
    gamepadState.value.leftStick.x !== 0 || gamepadState.value.leftStick.y !== 0
  );

  // Gamepad connection handlers
  function handleGamepadConnected(event: GamepadEvent) {
    if (gamepadIndex.value === null) {
      gamepadIndex.value = event.gamepad.index;
      gamepadState.value.connected = true;
      gamepadState.value.type = detectGamepadType(event.gamepad.id);
      console.log(`Gamepad connected: ${event.gamepad.id} (${gamepadState.value.type})`);
    }
  }

  function handleGamepadDisconnected(event: GamepadEvent) {
    if (gamepadIndex.value === event.gamepad.index) {
      gamepadIndex.value = null;
      gamepadState.value.connected = false;
      console.log('Gamepad disconnected');
    }
  }

  // Animation frame for continuous updates
  let animationFrameId: number;

  function startGamepadPolling() {
    function poll() {
      updateGamepad();
      animationFrameId = requestAnimationFrame(poll);
    }
    poll();
  }

  function stopGamepadPolling() {
    if (animationFrameId) {
      cancelAnimationFrame(animationFrameId);
    }
  }

  // Lifecycle hooks
  onMounted(() => {
    window.addEventListener('gamepadconnected', handleGamepadConnected);
    window.addEventListener('gamepaddisconnected', handleGamepadDisconnected);
    
    // Check for already connected gamepads
    const gamepads = navigator.getGamepads();
    for (let i = 0; i < gamepads.length; i++) {
      if (gamepads[i]) {
        gamepadIndex.value = i;
        gamepadState.value.connected = true;
        gamepadState.value.type = detectGamepadType(gamepads[i]!.id);
        break;
      }
    }
    
    startGamepadPolling();
  });

  onUnmounted(() => {
    window.removeEventListener('gamepadconnected', handleGamepadConnected);
    window.removeEventListener('gamepaddisconnected', handleGamepadDisconnected);
    stopGamepadPolling();
  });

  return {
    gamepadState,
    scrollSpeed,
    isMoving,
    onButtonPress,
  };
}