import { ref } from 'vue';

const copiedStatus = ref(new Map<string, boolean>());
const timeouts = ref(new Map<string, number>());

const COPIED_DURATION = 1500; // milliseconds

export function useCopyToClipboard() {

  const copy = async (identifier: string, textToCopy: string) => {
    if (!navigator.clipboard) {
      console.error('Clipboard API not available');
      return;
    }
    try {
      await navigator.clipboard.writeText(textToCopy);

      const existingTimeout = timeouts.value.get(identifier);
      if (existingTimeout) {
        clearTimeout(existingTimeout);
      }

      copiedStatus.value.set(identifier, true);
      copiedStatus.value = new Map(copiedStatus.value);


      // Set a timeout to revert the state
      const timeoutId = window.setTimeout(() => {
        copiedStatus.value.delete(identifier);
        timeouts.value.delete(identifier);
         // Force reactivity update
        copiedStatus.value = new Map(copiedStatus.value);
      }, COPIED_DURATION);

      timeouts.value.set(identifier, timeoutId);

    } catch (err) {
      console.error('Failed to copy text: ', err);
    }
  };

  const isCopied = (identifier: string): boolean => {
    return copiedStatus.value.get(identifier) ?? false;
  };

  const getIcon = (identifier: string, defaultIcon: string = '$clipboard-multiple-outline', copiedIcon: string = '$clipboard-check-multiple-outline'): string => {
    return isCopied(identifier) ? copiedIcon : defaultIcon;
  };

  const getColor = (identifier: string, copiedColor: string = 'green', defaultColor: string | undefined = undefined): string | undefined => {
    return isCopied(identifier) ? copiedColor : defaultColor;
  };

  return {
    copy,
    isCopied,
    getIcon,
    getColor,
  };
}
