import {computed, ref, type ComputedRef, type MaybeRef} from 'vue';

import {suivre as follow} from '@constl/vue';
import {onMounted} from 'vue';
import {useOrbiter} from '/@/plugins/orbiter/utils';

export const useUserProfilePhoto = (
  accountId?: MaybeRef<string | undefined>,
): ComputedRef<string | undefined> => {
  const {orbiter} = useOrbiter();

  const profilePic = follow(orbiter.listenForProfilePhotoChange.bind(orbiter), {accountId});
  const defaultAvatar = ref<string>();
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
