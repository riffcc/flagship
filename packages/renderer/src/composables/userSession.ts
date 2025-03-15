import { type Ref, ref } from 'vue';

export interface UserData {
  id: string;
  name: string;
  email: string;
}

const userData: Ref<UserData | null> = ref({id: '1', name: 'Jhon Doe', email: 'jhondoe@test.com'});

export const useUserSession = () => {
  return {
    userData,
  };
};
