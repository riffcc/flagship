import { ref, onMounted, onUnmounted } from 'vue';
import { useSpatialNavigation, type Direction } from './useSpatialNavigation';

// Shared state for quick search integration
const quickSearchOpen = ref(false);

/**
 * Keyboard navigation composable.
 * Enables arrow key navigation on content grids.
 */
export function useKeyboardNavigation() {
  const {
    navigate,
    activate,
    clearFocus,
    focusedElement,
    isLocked,
  } = useSpatialNavigation();

  /**
   * Check if the event target is an input element where we shouldn't intercept keys.
   */
  function isInputElement(target: EventTarget | null): boolean {
    if (!target) return false;
    const el = target as HTMLElement;
    const tagName = el.tagName?.toLowerCase();
    return (
      tagName === 'input' ||
      tagName === 'textarea' ||
      tagName === 'select' ||
      el.isContentEditable
    );
  }

  /**
   * Handle keydown events for navigation.
   */
  function handleKeydown(event: KeyboardEvent) {
    // Don't navigate when quick search is open or in input elements
    if (quickSearchOpen.value) return;
    if (isInputElement(event.target)) return;

    // Arrow key navigation
    const directionMap: Record<string, Direction> = {
      ArrowUp: 'up',
      ArrowDown: 'down',
      ArrowLeft: 'left',
      ArrowRight: 'right',
    };

    const direction = directionMap[event.key];
    if (direction) {
      event.preventDefault();
      navigate(direction);
      return;
    }

    // Enter to activate
    if (event.key === 'Enter' && focusedElement.value) {
      event.preventDefault();
      activate();
      return;
    }
  }

  /**
   * Clear focus on mouse click (switch to mouse mode).
   */
  function handleMouseClick() {
    clearFocus();
  }

  onMounted(() => {
    document.addEventListener('keydown', handleKeydown);
    document.addEventListener('click', handleMouseClick);
  });

  onUnmounted(() => {
    document.removeEventListener('keydown', handleKeydown);
    document.removeEventListener('click', handleMouseClick);
  });

  return {
    focusedElement,
    quickSearchOpen,
    setQuickSearchOpen: (open: boolean) => {
      quickSearchOpen.value = open;
    },
  };
}

/**
 * Set quick search open state from outside the composable.
 * Used by quick search to disable keyboard navigation when open.
 */
export function setQuickSearchOpen(open: boolean) {
  quickSearchOpen.value = open;
}

export { quickSearchOpen };
