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
    const svg = await [
      import('/@/assets/undraw/undraw_pic_profile_re_7g2h.svg'),
      import('/@/assets/undraw/undraw_profile_pic_re_iwgo.svg'),
    ][Math.floor(Math.random() * 2)];
    userData.value = {
      id: '1',
      name: 'Jhon Doe',
      email: 'jhondoe@test.com',
      avatar: svg.default,
    };
  });


  return {
    userData,
  };
};
