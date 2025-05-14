import {computed, ref, watchEffect, type ComputedRef, type MaybeRef, unref} from 'vue';
import {onMounted} from 'vue';
// import {useOrbiter} from '/@/plugins/orbiter/utils'; // Removed

export const useUserProfilePhoto = (
  accountId?: MaybeRef<string | undefined>,
): ComputedRef<string | undefined> => {
  // const {orbiter} = useOrbiter(); // Removed

  const profilePic = ref<Uint8Array | undefined>(undefined);
  const defaultAvatar = ref<string>();

  watchEffect((onCleanup) => {
    const currentAccountId = unref(accountId);
    // Removed orbiter dependent logic for profilePic
    // if (orbiter && orbiter.listenForProfilePhotoChange && currentAccountId) { ... }
    // else {
    profilePic.value = undefined; // Reset if no accountId or orbiter method, or keep default behavior
    // }
  });

  onMounted(async () => {
    const svg = await [
      import('/@/assets/undraw/undraw_pic_profile_re_7g2h.svg'),
      import('/@/assets/undraw/undraw_profile_pic_re_iwgo.svg'),
    ][Math.floor(Math.random() * 2)]; // Let's keep it fair and random :)
    defaultAvatar.value = svg.default;
  });

  const profilePicSrc = computed(() => {
    if (profilePic.value) {
      return URL.createObjectURL(new Blob([profilePic.value], {type: 'image'}));
    } else {
      return defaultAvatar.value;
    }
  });
  return profilePicSrc;
};
