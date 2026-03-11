import { type App } from 'vue';
import { CitadelService } from '@riffcc/citadel-sdk';
import { getApiUrl } from '/@/utils/runtimeConfig';

export * from './hooks';

export default {
  install: (app: App) => {
    // Always use CitadelService for HTTP-based API access
    const apiUrl = getApiUrl();
    const lensServiceInstance = new CitadelService(apiUrl);

    app.provide('lensService', lensServiceInstance);
    app.config.globalProperties.$lensService = lensServiceInstance;
  },
};
