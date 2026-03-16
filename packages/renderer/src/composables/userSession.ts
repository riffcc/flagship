import { type Ref, ref } from 'vue';

export interface UserData {
  id: string;
  name: string;
  email: string;
  avatar: string;
  bio?: string;
}

const userData: Ref<UserData | null> = ref(null);
const USER_DATA_STORAGE_KEY = 'riff-user-session';

function getDefaultUserData(): UserData {
  return {
    id: '1',
    name: 'Test User',
    email: 'testing@riff.cc',
    avatar: '',
    bio: '',
  };
}

function ensureUserData(): UserData {
  if (userData.value) {
    return userData.value;
  }

  if (typeof window === 'undefined') {
    userData.value = getDefaultUserData();
    return userData.value;
  }

  try {
    const stored = window.localStorage.getItem(USER_DATA_STORAGE_KEY);
    if (stored) {
      userData.value = {
        ...getDefaultUserData(),
        ...JSON.parse(stored) as Partial<UserData>,
      };
      return userData.value;
    }
  } catch (error) {
    console.warn('[useUserSession] Failed to load stored user data:', error);
  }

  userData.value = getDefaultUserData();
  return userData.value;
}

function persistUserData() {
  if (typeof window === 'undefined' || !userData.value) {
    return;
  }

  try {
    window.localStorage.setItem(USER_DATA_STORAGE_KEY, JSON.stringify(userData.value));
  } catch (error) {
    console.warn('[useUserSession] Failed to persist user data:', error);
  }
}

export const useUserSession = () => {
  ensureUserData();

  function updateUserData(patch: Partial<UserData>) {
    const current = ensureUserData();
    userData.value = {
      ...current,
      ...patch,
    };
    persistUserData();
  }

  function clearUserData() {
    userData.value = null;
    if (typeof window !== 'undefined') {
      window.localStorage.removeItem(USER_DATA_STORAGE_KEY);
    }
  }

  return {
    userData,
    updateUserData,
    clearUserData,
  };
};
