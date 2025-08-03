import { ref, watch, onMounted, onUnmounted, type Ref } from 'vue';
import { useRouter } from 'vue-router';
import { useGamepad } from './useGamepad';
import { useAudioAlbum } from './audioAlbum';
import { useFloatingVideo } from './floatingVideo';

interface NavigableElement {
  element: HTMLElement;
  x: number;
  y: number;
  width: number;
  height: number;
}

// Navigation lock for modal dialogs
const navigationLocked = ref(false);

export function useGamepadNavigation() {
  const router = useRouter();
  const { gamepadState, onButtonPress } = useGamepad();
  
  const focusedElement = ref<HTMLElement | null>(null);
  const navigableElements = ref<NavigableElement[]>([]);
  const lastStickDirection = ref({ x: 0, y: 0 });
  const lastDpadState = ref({ up: false, down: false, left: false, right: false });
  
  // Custom cursor for right stick
  const cursorPosition = ref({ x: window.innerWidth / 2, y: window.innerHeight / 2 });
  const showCursor = ref(false);
  
  // Track which control method was used last
  const lastControlMethod = ref<'leftStick' | 'rightStick'>('leftStick');
  
  // Cursor timeout
  let cursorTimeoutId: number | undefined;
  
  // Tab navigation
  const currentTabIndex = ref(0);
  const tabs = ['/', '/featured/music', '/featured/movies', '/featured/tv-shows'];
  
  // Update navigable elements
  function updateNavigableElements() {
    // If navigation is locked, only look for elements in the modal
    if (navigationLocked.value) {
      const elements = document.querySelectorAll('.start-menu-card [data-navigable="true"], .start-menu-card .v-list-item');
      navigableElements.value = Array.from(elements).map(el => {
        const rect = (el as HTMLElement).getBoundingClientRect();
        return {
          element: el as HTMLElement,
          x: rect.left + rect.width / 2,
          y: rect.top + rect.height / 2,
          width: rect.width,
          height: rect.height,
        };
      });
    } else {
      const elements = document.querySelectorAll('[data-navigable="true"], a[href], button:not(:disabled), [role="button"], .content-card[cursor-pointer], .v-btn:not(:disabled), .v-list-item');
      navigableElements.value = Array.from(elements)
        .filter(el => {
          const element = el as HTMLElement;
          // Filter out non-interactive elements
          if (element.tagName === 'DIV' && !element.classList.contains('content-card') && !element.hasAttribute('data-navigable')) {
            return false;
          }
          // Make sure element is visible
          const rect = element.getBoundingClientRect();
          return rect.width > 0 && rect.height > 0;
        })
        .map(el => {
          const rect = (el as HTMLElement).getBoundingClientRect();
          return {
            element: el as HTMLElement,
            x: rect.left + rect.width / 2,
            y: rect.top + rect.height / 2,
            width: rect.width,
            height: rect.height,
          };
        });
    }
  }
  
  // Find nearest element in a direction
  function findNearestElement(direction: 'up' | 'down' | 'left' | 'right'): HTMLElement | null {
    if (!focusedElement.value) {
      return navigableElements.value[0]?.element || null;
    }
    
    const currentRect = focusedElement.value.getBoundingClientRect();
    const currentX = currentRect.left + currentRect.width / 2;
    const currentY = currentRect.top + currentRect.height / 2;
    
    let candidates = navigableElements.value.filter(el => {
      switch (direction) {
        case 'up':
          return el.y < currentY - 10;
        case 'down':
          return el.y > currentY + 10;
        case 'left':
          return el.x < currentX - 10;
        case 'right':
          return el.x > currentX + 10;
      }
    });
    
    // Sort by distance
    candidates.sort((a, b) => {
      const distA = Math.sqrt(Math.pow(a.x - currentX, 2) + Math.pow(a.y - currentY, 2));
      const distB = Math.sqrt(Math.pow(b.x - currentX, 2) + Math.pow(b.y - currentY, 2));
      return distA - distB;
    });
    
    return candidates[0]?.element || null;
  }
  
  // Focus an element with visual feedback
  function focusElement(element: HTMLElement | null) {
    // Remove previous focus
    if (focusedElement.value) {
      focusedElement.value.classList.remove('gamepad-focused');
      focusedElement.value.style.removeProperty('box-shadow');
      focusedElement.value.style.removeProperty('outline');
    }
    
    if (element) {
      element.focus();
      element.classList.add('gamepad-focused');
      // Purple neon glow for TV viewing
      element.style.boxShadow = '0 0 0 3px rgba(138, 43, 226, 0.8), 0 0 20px rgba(138, 43, 226, 0.5)';
      element.style.outline = 'none';
      element.scrollIntoView({ behavior: 'smooth', block: 'center' });
      focusedElement.value = element;
    }
  }
  
  // Handle left stick navigation
  watch([() => gamepadState.value.leftStick, () => gamepadState.value.buttons], ([stick, buttons]) => {
    // Skip navigation if locked (modal is open)
    if (navigationLocked.value) return;
    
    // Update control method and hide cursor when using left stick
    if (Math.abs(stick.x) > 0.3 || Math.abs(stick.y) > 0.3 || 
        buttons.up || buttons.down || buttons.left || buttons.right) {
      lastControlMethod.value = 'leftStick';
      showCursor.value = false;
      const cursorEl = document.getElementById('gamepad-cursor');
      if (cursorEl) {
        cursorEl.style.display = 'none';
      }
    }
    
    // D-pad navigation (instant snap)
    if (buttons.up) {
      const element = findNearestElement('up');
      if (element) {
        focusElement(element);
        lastNavigationTime.value = now;
      }
    } else if (buttons.down) {
      const element = findNearestElement('down');
      if (element) {
        focusElement(element);
        lastNavigationTime.value = now;
      }
    } else if (buttons.left) {
      const element = findNearestElement('left');
      if (element) {
        focusElement(element);
        lastNavigationTime.value = now;
      }
    } else if (buttons.right) {
      const element = findNearestElement('right');
      if (element) {
        focusElement(element);
        lastNavigationTime.value = now;
      }
    }
    
    // Left stick navigation (snap to elements, no free movement)
    if (Math.abs(stick.x) > 0.6 || Math.abs(stick.y) > 0.6) {
      let direction: 'up' | 'down' | 'left' | 'right';
      
      // Determine primary direction with stronger threshold
      if (Math.abs(stick.x) > Math.abs(stick.y)) {
        direction = stick.x > 0 ? 'right' : 'left';
      } else {
        direction = stick.y > 0 ? 'down' : 'up';
      }
      
      const element = findNearestElement(direction);
      if (element) {
        focusElement(element);
        lastNavigationTime.value = now;
      }
    }
  });
  
  // Handle right stick cursor
  watch(() => gamepadState.value.rightStick, (stick) => {
    if (Math.abs(stick.x) > 0.1 || Math.abs(stick.y) > 0.1) {
      lastControlMethod.value = 'rightStick';
      showCursor.value = true;
      const speed = 10;
      cursorPosition.value.x = Math.max(0, Math.min(window.innerWidth, cursorPosition.value.x + stick.x * speed));
      // Normal Y axis - positive stick.y moves cursor down
      cursorPosition.value.y = Math.max(0, Math.min(window.innerHeight, cursorPosition.value.y + stick.y * speed));
      
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
  });
  
  // Button handlers
  onButtonPress('a', () => {
    // Select based on which control method was used last
    if (lastControlMethod.value === 'rightStick' && showCursor.value) {
      // Click at cursor position if right stick was used last
      const element = document.elementFromPoint(cursorPosition.value.x, cursorPosition.value.y) as HTMLElement;
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
  
  // Removed - R3 is now used for play/pause
  
  // Start button is handled in App.vue for the start menu overlay
  // onButtonPress('start', () => {
  //   // Open menu
  //   router.push('/menu');
  // });
  
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
  
  // Update navigable elements on route change and mutations
  const observer = new MutationObserver(() => {
    updateNavigableElements();
  });
  
  // Clear focus on mouse click
  function handleMouseClick() {
    if (focusedElement.value) {
      focusedElement.value.classList.remove('gamepad-focused');
      focusedElement.value.style.removeProperty('box-shadow');
      focusedElement.value.style.removeProperty('outline');
      focusedElement.value = null;
    }
  }
  
  onMounted(() => {
    updateNavigableElements();
    observer.observe(document.body, { childList: true, subtree: true });
    
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
    observer.disconnect();
    document.removeEventListener('click', handleMouseClick);
    if (cursorTimeoutId) {
      clearTimeout(cursorTimeoutId);
    }
    const cursor = document.getElementById('gamepad-cursor');
    cursor?.remove();
  });
  
  // Lock/unlock navigation for modal dialogs
  function lockNavigation() {
    navigationLocked.value = true;
    updateNavigableElements();
  }
  
  function unlockNavigation() {
    navigationLocked.value = false;
    updateNavigableElements();
  }

  return {
    focusedElement,
    showCursor,
    cursorPosition,
    updateNavigableElements,
    lockNavigation,
    unlockNavigation,
  };
}