import { ref } from 'vue';

type MessageType = 'success' | 'error';

export function useSnackbarMessage() {

  const message = ref<{ text: string; type: MessageType} | undefined>();
  const show = ref(false);

  const open = (text: string, type: MessageType) => {
    message.value = {
      text,
      type,
    };
    show.value = true;
  };
  const close = () => {
    show.value = false;
    message.value = undefined;
  };
  return {
    snackbarMessage: message,
    showSnackbar: show,
    openSnackbar: open,
    closeSnackbar: close,
  };
}
