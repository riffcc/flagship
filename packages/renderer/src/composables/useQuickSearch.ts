import { ref, onMounted, onUnmounted } from 'vue';
import { setQuickSearchOpen } from './useKeyboardNavigation';

// Shared state
const isOpen = ref(false);
const query = ref('');

/**
 * Quick search composable.
 * Enables "just type" functionality - start typing anywhere to open search.
 */
export function useQuickSearch() {
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
   * Open quick search.
   */
  function open(initialChar?: string) {
    isOpen.value = true;
    setQuickSearchOpen(true);
    if (initialChar) {
      query.value = initialChar;
    }
  }

  /**
   * Close quick search.
   */
  function close() {
    isOpen.value = false;
    query.value = '';
    setQuickSearchOpen(false);
  }

  /**
   * Handle global keydown for "just type" activation.
   */
  function handleKeydown(event: KeyboardEvent) {
    // Don't intercept if already in an input element
    if (isInputElement(event.target)) return;

    // If quick search is open, handle escape and backspace
    if (isOpen.value) {
      if (event.key === 'Escape') {
        event.preventDefault();
        close();
        return;
      }
      // Backspace when empty closes quick search
      if (event.key === 'Backspace' && query.value.length === 0) {
        event.preventDefault();
        close();
        return;
      }
      // Let the input handle other keys
      return;
    }

    // Open quick search on alphanumeric key press
    if (
      event.key.length === 1 &&
      !event.ctrlKey &&
      !event.metaKey &&
      !event.altKey &&
      /^[a-zA-Z0-9]$/.test(event.key)
    ) {
      event.preventDefault();
      open(event.key);
    }
  }

  onMounted(() => {
    document.addEventListener('keydown', handleKeydown);
  });

  onUnmounted(() => {
    document.removeEventListener('keydown', handleKeydown);
  });

  return {
    isOpen,
    query,
    open,
    close,
  };
}

// Export for direct access
export { isOpen as quickSearchIsOpen, query as quickSearchQuery };
