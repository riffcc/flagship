import { ref, onMounted, onUnmounted } from 'vue';

export type InputMethod = 'mouse' | 'gamepad' | 'keyboard';

const currentInputMethod = ref<InputMethod>('mouse');
let lastMousePosition = { x: 0, y: 0 };
let mouseCheckInterval: number | undefined;

export function useInputMethod() {
  function setInputMethod(method: InputMethod) {
    if (currentInputMethod.value !== method) {
      currentInputMethod.value = method;
      updateBodyClass(method);
    }
  }

  function updateBodyClass(method: InputMethod) {
    document.body.classList.remove('input-mouse', 'input-gamepad', 'input-keyboard');
    document.body.classList.add(`input-${method}`);
  }

  // Detect mouse movement
  function handleMouseMove(e: MouseEvent) {
    const moved = Math.abs(e.clientX - lastMousePosition.x) > 5 || 
                  Math.abs(e.clientY - lastMousePosition.y) > 5;
    
    if (moved) {
      lastMousePosition = { x: e.clientX, y: e.clientY };
      setInputMethod('mouse');
    }
  }

  // Detect keyboard input
  function handleKeyboard(e: KeyboardEvent) {
    // Ignore gamepad-simulated keyboard events
    if (!e.isTrusted) return;
    setInputMethod('keyboard');
  }

  onMounted(() => {
    // Initialize body class
    updateBodyClass(currentInputMethod.value);
    
    // Add event listeners
    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('keydown', handleKeyboard);
    
    // Store initial mouse position
    lastMousePosition = { x: window.innerWidth / 2, y: window.innerHeight / 2 };
  });

  onUnmounted(() => {
    document.removeEventListener('mousemove', handleMouseMove);
    document.removeEventListener('keydown', handleKeyboard);
    if (mouseCheckInterval) {
      clearInterval(mouseCheckInterval);
    }
  });

  return {
    currentInputMethod,
    setInputMethod,
  };
}