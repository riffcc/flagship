import { ref, watch } from 'vue';
import { useTheme as useVuetifyTheme } from 'vuetify';

const THEME_KEY = 'riffcc-theme';

export function useTheme() {
  const vuetifyTheme = useVuetifyTheme();

  // Load theme from localStorage or default to dark
  const storedTheme = localStorage.getItem(THEME_KEY);
  const isDark = ref(storedTheme ? storedTheme === 'dark' : true);

  // Apply initial theme
  vuetifyTheme.global.name.value = isDark.value ? 'dark' : 'light';

  // Watch for theme changes and persist
  watch(isDark, (newValue) => {
    vuetifyTheme.global.name.value = newValue ? 'dark' : 'light';
    localStorage.setItem(THEME_KEY, newValue ? 'dark' : 'light');
  });

  const toggleTheme = () => {
    isDark.value = !isDark.value;
  };

  const setTheme = (theme: 'dark' | 'light') => {
    isDark.value = theme === 'dark';
  };

  return {
    isDark,
    toggleTheme,
    setTheme,
    currentTheme: vuetifyTheme.global.current,
  };
}
