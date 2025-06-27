import { reactive } from 'vue';
import { lensColorHash } from '/@/utils';

const selectedColors = reactive<Record<string, string>>({});

export function useSiteColors() {
  const getSiteColor = (siteAddress: string): string => {
    if (selectedColors[siteAddress]) {
      return selectedColors[siteAddress];
    }

    const storedColor = localStorage.getItem(`siteColor_${siteAddress}`);
    if (storedColor) {
      selectedColors[siteAddress] = storedColor;
      return storedColor;
    }

    const defaultColor = lensColorHash(siteAddress);
    selectedColors[siteAddress] = defaultColor;
    localStorage.setItem(`siteColor_${siteAddress}`, defaultColor);
    return defaultColor;
  };

  const saveColor = (siteAddress: string, color: string) => {
    if (color) {
      selectedColors[siteAddress] = color;
      localStorage.setItem(`siteColor_${siteAddress}`, color);
    }
  };

  return {
    selectedColors,
    getSiteColor,
    saveColor,
  };
}
