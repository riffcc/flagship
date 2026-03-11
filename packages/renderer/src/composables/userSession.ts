import { onMounted, type Ref, ref } from 'vue';

export interface UserData {
  id: string;
  name: string;
  email: string;
  avatar: string;
}

const userData: Ref<UserData | null> = ref(null);

export const useUserSession = () => {
  onMounted(async () => {
    userData.value = {
      id: '1',
      name: 'Test User',
      email: 'testing@riff.cc',
      avatar: '', // Let the accountMenu component handle the default avatar
    };
  });


  return {
    userData,
  };
};
