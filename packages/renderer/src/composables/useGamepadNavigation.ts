import { ref, watch, onMounted, onUnmounted } from 'vue';
import { useRouter } from 'vue-router';
import { useGamepad } from './useGamepad';
import { useAudioAlbum } from './audioAlbum';
import { useFloatingVideo } from './floatingVideo';
import {
  useSpatialNavigation,
  type Direction,
} from './useSpatialNavigation';

export function useGamepadNavigation() {
  const router = useRouter();
  const { gamepadState, onButtonPress } = useGamepad();

  // Use shared spatial navigation
  const {
    focusedElement,
    navigate,
    focusElement,
    clearFocus,
    lockNavigation,
    unlockNavigation,
    updateNavigableElements,
    isLocked,
  } = useSpatialNavigation();

  // Track which control method was used last
  const lastControlMethod = ref<'leftStick' | 'rightStick'>('leftStick');

  // Custom cursor for right stick
  const cursorPosition = ref({ x: window.innerWidth / 2, y: window.innerHeight / 2 });
  const showCursor = ref(false);

  // Cursor timeout
  let cursorTimeoutId: number | undefined;

  // Tab navigation
  const currentTabIndex = ref(0);
  const tabs = ['/', '/music', '/movies', '/tv'];

  // Handle left stick navigation
  watch([() => gamepadState.value.leftStick, () => gamepadState.value.buttons], ([stick, buttons]) => {
    // Skip navigation if locked (modal is open)
    if (isLocked()) return;

    // Update control method and hide cursor when using left stick
    if (
      Math.abs(stick.x) > 0.3 ||
      Math.abs(stick.y) > 0.3 ||
      buttons.up ||
      buttons.down ||
      buttons.left ||
      buttons.right
    ) {
      lastControlMethod.value = 'leftStick';
      showCursor.value = false;
      const cursorEl = document.getElementById('gamepad-cursor');
      if (cursorEl) {
        cursorEl.style.display = 'none';
      }
    }

    // D-pad navigation (instant snap)
    if (buttons.up) {
      navigate('up');
    } else if (buttons.down) {
      navigate('down');
    } else if (buttons.left) {
      navigate('left');
    } else if (buttons.right) {
      navigate('right');
    }
    // Left stick navigation (snap to elements, no free movement)
    else if (Math.abs(stick.x) > 0.6 || Math.abs(stick.y) > 0.6) {
      let direction: Direction;

      // Determine primary direction with stronger threshold
      if (Math.abs(stick.x) > Math.abs(stick.y)) {
        direction = stick.x > 0 ? 'right' : 'left';
      } else {
        direction = stick.y > 0 ? 'down' : 'up';
      }

      navigate(direction);
    }
  });

  // Handle right stick cursor
  watch(
    () => gamepadState.value.rightStick,
    stick => {
      if (Math.abs(stick.x) > 0.1 || Math.abs(stick.y) > 0.1) {
        lastControlMethod.value = 'rightStick';
        showCursor.value = true;
        const speed = 10;
        cursorPosition.value.x = Math.max(
          0,
          Math.min(window.innerWidth, cursorPosition.value.x + stick.x * speed)
        );
        // Normal Y axis - positive stick.y moves cursor down
        cursorPosition.value.y = Math.max(
          0,
          Math.min(window.innerHeight, cursorPosition.value.y + stick.y * speed)
        );

        // Update cursor element position
        const cursorEl = document.getElementById('gamepad-cursor');
        if (cursorEl) {
          cursorEl.style.display = 'block';
          cursorEl.style.opacity = '1';
          cursorEl.style.left = `${cursorPosition.value.x}px`;
          cursorEl.style.top = `${cursorPosition.value.y}px`;
        }

        // Clear existing timeout and set new one
        if (cursorTimeoutId) {
          clearTimeout(cursorTimeoutId);
        }

        // Fade cursor after 4 seconds of inactivity
        cursorTimeoutId = setTimeout(() => {
          const cursorEl = document.getElementById('gamepad-cursor');
          if (cursorEl) {
            cursorEl.style.transition = 'opacity 1s ease-out';
            cursorEl.style.opacity = '0';
            setTimeout(() => {
              cursorEl.style.display = 'none';
              showCursor.value = false;
            }, 1000);
          }
        }, 4000) as unknown as number;
      }
    }
  );

  // Button handlers
  onButtonPress('a', () => {
    // Select based on which control method was used last
    if (lastControlMethod.value === 'rightStick' && showCursor.value) {
      // Click at cursor position if right stick was used last
      const element = document.elementFromPoint(
        cursorPosition.value.x,
        cursorPosition.value.y
      ) as HTMLElement;
      if (element && typeof element.click === 'function') {
        element.click();
      }
    } else if (focusedElement.value && typeof focusedElement.value.click === 'function') {
      // Click focused element if left stick was used last
      focusedElement.value.click();
    }
  });

  onButtonPress('b', () => {
    router.back();
  });

  onButtonPress('x', () => {
    // Stop any playing media
    const { activeTrack } = useAudioAlbum();
    const { floatingVideoSource, closeFloatingVideo } = useFloatingVideo();

    if (activeTrack.value) {
      activeTrack.value = undefined;
    }
    if (floatingVideoSource.value) {
      closeFloatingVideo();
    }
  });

  onButtonPress('y', () => {
    // Toggle search/filter panel
    const filterPanel = document.querySelector('[data-filter-panel]');
    if (filterPanel) {
      filterPanel.classList.toggle('show');
    } else {
      // Navigate to search if no filter panel present
      router.push('/search');
    }
  });

  onButtonPress('lb', () => {
    currentTabIndex.value = Math.max(0, currentTabIndex.value - 1);
    router.push(tabs[currentTabIndex.value]);
  });

  onButtonPress('rb', () => {
    currentTabIndex.value = Math.min(tabs.length - 1, currentTabIndex.value + 1);
    router.push(tabs[currentTabIndex.value]);
  });

  // L3/R3 for play/pause
  const playPauseHandler = () => {
    const { activeTrack } = useAudioAlbum();
    const audioPlayer = document.querySelector('audio') as HTMLAudioElement;
    const videoPlayer = document.querySelector('video') as HTMLVideoElement;

    if (audioPlayer) {
      if (audioPlayer.paused) {
        audioPlayer.play();
      } else {
        audioPlayer.pause();
      }
    } else if (videoPlayer) {
      if (videoPlayer.paused) {
        videoPlayer.play();
      } else {
        videoPlayer.pause();
      }
    } else if (focusedElement.value) {
      // If no media is playing, try to click the focused element (e.g., play button)
      focusedElement.value.click();
    }
  };

  onButtonPress('leftStickButton', playPauseHandler);
  onButtonPress('rightStickButton', playPauseHandler);

  // Clear focus on mouse click
  function handleMouseClick() {
    clearFocus();
  }

  onMounted(() => {
    // Add global mouse click handler
    document.addEventListener('click', handleMouseClick);

    // Create cursor element
    const cursor = document.createElement('div');
    cursor.id = 'gamepad-cursor';
    cursor.style.cssText = `
      position: fixed;
      width: 24px;
      height: 24px;
      border: 3px solid #8a2be2;
      border-radius: 50%;
      background: rgba(138, 43, 226, 0.3);
      pointer-events: none;
      z-index: 10000;
      display: none;
      transform: translate(-50%, -50%);
      opacity: 1;
    `;
    document.body.appendChild(cursor);
  });

  onUnmounted(() => {
    document.removeEventListener('click', handleMouseClick);
    if (cursorTimeoutId) {
      clearTimeout(cursorTimeoutId);
    }
    const cursor = document.getElementById('gamepad-cursor');
    cursor?.remove();
  });

  return {
    focusedElement,
    showCursor,
    cursorPosition,
    updateNavigableElements,
    lockNavigation,
    unlockNavigation,
  };
}
