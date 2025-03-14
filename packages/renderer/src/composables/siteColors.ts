import { reactive } from 'vue';
import { lensColorHash } from '/@/utils';

const selectedColors = reactive<Record<string, string>>({});

export function useSiteColors() {
  const getSiteColor = (siteId: string): string => {
    if (selectedColors[siteId]) {
      return selectedColors[siteId];
    }

    const storedColor = localStorage.getItem(`siteColor_${siteId}`);
    if (storedColor) {
      selectedColors[siteId] = storedColor;
      return storedColor;
    }

    const defaultColor = lensColorHash(siteId);
    selectedColors[siteId] = defaultColor;
    localStorage.setItem(`siteColor_${siteId}`, defaultColor);
    return defaultColor;
  };

  const saveColor = (siteId: string, color: string) => {
    if (color) {
      selectedColors[siteId] = color;
      localStorage.setItem(`siteColor_${siteId}`, color);
    }
  };

  return {
    selectedColors,
    getSiteColor,
    saveColor,
  };
}
