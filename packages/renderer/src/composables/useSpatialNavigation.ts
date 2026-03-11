import { ref, onMounted, onUnmounted } from 'vue';

export interface NavigableElement {
  element: HTMLElement;
  x: number;
  y: number;
  width: number;
  height: number;
}

export type Direction = 'up' | 'down' | 'left' | 'right';

// Shared state for navigation lock (modal dialogs)
const navigationLocked = ref(false);

// Currently focused element (shared across all navigation methods)
const focusedElement = ref<HTMLElement | null>(null);

// Navigable elements cache
const navigableElements = ref<NavigableElement[]>([]);

// Navigation cooldown
const lastNavigationTime = ref(0);
const NAVIGATION_COOLDOWN = 200; // ms

/**
 * Shared spatial navigation composable.
 * Provides core navigation logic reusable by keyboard, gamepad, and other input methods.
 */
export function useSpatialNavigation() {
  // MutationObserver for tracking DOM changes
  let observer: MutationObserver | null = null;

  /**
   * Update the list of navigable elements on the page.
   */
  function updateNavigableElements() {
    if (navigationLocked.value) {
      // When locked (modal open), only look for elements in the modal
      const elements = document.querySelectorAll(
        '.start-menu-card [data-navigable="true"], .start-menu-card .v-list-item, [data-modal] [data-navigable="true"]'
      );
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
      const elements = document.querySelectorAll(
        '[data-navigable="true"], a[href], button:not(:disabled), [role="button"], .content-card[cursor-pointer], .v-btn:not(:disabled), .v-list-item'
      );
      navigableElements.value = Array.from(elements)
        .filter(el => {
          const element = el as HTMLElement;
          // Filter out non-interactive elements
          if (
            element.tagName === 'DIV' &&
            !element.classList.contains('content-card') &&
            !element.hasAttribute('data-navigable')
          ) {
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

  /**
   * Find the nearest navigable element in a given direction.
   */
  function findNearestElement(direction: Direction): HTMLElement | null {
    updateNavigableElements();

    if (!focusedElement.value) {
      return navigableElements.value[0]?.element || null;
    }

    const currentRect = focusedElement.value.getBoundingClientRect();
    const currentX = currentRect.left + currentRect.width / 2;
    const currentY = currentRect.top + currentRect.height / 2;

    const candidates = navigableElements.value.filter(el => {
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

  /**
   * Apply focus styling to an element.
   */
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
      // Purple neon glow for TV/keyboard viewing
      element.style.boxShadow =
        '0 0 0 3px rgba(138, 43, 226, 0.8), 0 0 20px rgba(138, 43, 226, 0.5)';
      element.style.outline = 'none';
      element.scrollIntoView({ behavior: 'smooth', block: 'center' });
      focusedElement.value = element;
    } else {
      focusedElement.value = null;
    }
  }

  /**
   * Navigate in a direction with cooldown protection.
   * Returns true if navigation occurred.
   */
  function navigate(direction: Direction): boolean {
    const now = Date.now();
    if (now - lastNavigationTime.value < NAVIGATION_COOLDOWN) {
      return false;
    }

    const element = findNearestElement(direction);
    if (element) {
      focusElement(element);
      lastNavigationTime.value = now;
      return true;
    }
    return false;
  }

  /**
   * Activate (click) the currently focused element.
   */
  function activate(): boolean {
    if (focusedElement.value && typeof focusedElement.value.click === 'function') {
      focusedElement.value.click();
      return true;
    }
    return false;
  }

  /**
   * Clear focus from current element.
   */
  function clearFocus() {
    if (focusedElement.value) {
      focusedElement.value.classList.remove('gamepad-focused');
      focusedElement.value.style.removeProperty('box-shadow');
      focusedElement.value.style.removeProperty('outline');
      focusedElement.value = null;
    }
  }

  /**
   * Lock navigation to modal elements only.
   */
  function lockNavigation() {
    navigationLocked.value = true;
    updateNavigableElements();
  }

  /**
   * Unlock navigation to all elements.
   */
  function unlockNavigation() {
    navigationLocked.value = false;
    updateNavigableElements();
  }

  /**
   * Check if navigation is currently locked.
   */
  function isLocked(): boolean {
    return navigationLocked.value;
  }

  /**
   * Initialize the mutation observer for DOM changes.
   */
  function initObserver() {
    if (observer) return;

    observer = new MutationObserver(() => {
      updateNavigableElements();
    });

    observer.observe(document.body, { childList: true, subtree: true });
    updateNavigableElements();
  }

  /**
   * Cleanup the mutation observer.
   */
  function destroyObserver() {
    if (observer) {
      observer.disconnect();
      observer = null;
    }
  }

  // Lifecycle hooks for components that use this
  onMounted(() => {
    initObserver();
  });

  onUnmounted(() => {
    destroyObserver();
  });

  return {
    // State
    focusedElement,
    navigableElements,
    navigationLocked,

    // Core functions
    navigate,
    findNearestElement,
    focusElement,
    activate,
    clearFocus,

    // Navigation lock
    lockNavigation,
    unlockNavigation,
    isLocked,

    // DOM observation
    updateNavigableElements,
    initObserver,
    destroyObserver,
  };
}
