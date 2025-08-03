import type { NavigationGuardNext, RouteLocationNormalized } from 'vue-router';
import { createRouter, createWebHashHistory, type RouteRecordRaw } from 'vue-router';
import { multiaddr } from '@multiformats/multiaddr';
// Keep HomePage as direct import since it's the landing page
import HomePage from '/@/views/homePage.vue';
import { queryClient } from './tanstackQuery';
import type { AccountStatusResponse } from '@riffcc/lens-sdk';

// Lazy load all other routes
const AdminPage = () => import('../views/adminPage.vue');
const AboutPage = () => import('/@/views/aboutPage.vue');
const AccountPage = () => import('/@/views/accountPage.vue');
const PrivacyPolicyPage = () => import('/@/views/privacyPolicyPage.vue');
const ReleasePage = () => import('/@/views/releasePage.vue');
const TermsPage = () => import('/@/views/termsPage.vue');
const UploadPage = () => import('/@/views/uploadPage.vue');
const CategoryPage = () => import('../views/categoryPage.vue');
const SeriesPage = () => import('../views/seriesPage.vue');

/**
 * Parses the VITE_LENS_NODE environment variable to build the base API URL.
 * This provides a single, reliable source of truth for the API endpoint.
 *
 * @returns {string} The constructed base API URL.
 */
function getApiUrl(): string {
  const lensNodeMaStr = import.meta.env.VITE_LENS_NODE;

  if (!lensNodeMaStr) {
    console.error('VITE_LENS_NODE environment variable is not set. API calls will fail.');
    // Fallback to a default or return an empty string, depending on desired behavior.
    return 'http://localhost:5002/api/v1';
  }

  try {
    const ma = multiaddr(lensNodeMaStr);
    const nodeOptions = ma.nodeAddress(); // Gets { family, address, port }

    // Determine the protocol based on the presence of 'wss' or 'ws'
    // 'wss' implies 'https' and 'ws' implies 'http'
    const protocol = ma.getComponents().map(c => c.name).includes('wss') ? 'https' : 'http';

    // Determine the API port. Conventionally, it might be different from the P2P port.
    // Let's assume a convention: P2P port 8002 -> API port 5002, P2P 4003 -> API 9002
    let apiPort;
    switch (nodeOptions.port) {
      case 8002: // Local P2P port
        apiPort = 5002;
        break;
      case 4003: // Production P2P port
        apiPort = 9002;
        break;
      default:
        // Default fallback if the P2P port is something unexpected
        console.warn(`Unexpected P2P port ${nodeOptions.port}, defaulting API port to 9002.`);
        apiPort = 9002;
    }

    return `${protocol}://${nodeOptions.address}:${apiPort}/api/v1`;

  } catch (error) {
    console.error('Failed to parse VITE_LENS_NODE multiaddr:', error);
    // Fallback if parsing fails
    return 'http://localhost:5002/api/v1';
  }
}

export const API_URL = getApiUrl();
const PREFETCH_CONFIG = {
  initialReleases: {
    url: `${API_URL}/releases`,
    queryKey: ['releases'],
  },
  initialFeaturedReleases: {
    url: `${API_URL}/featured-releases`,
    queryKey: ['featuredReleases'],
  },
  initialContentCategories: {
    url: `${API_URL}/content-categories`,
    queryKey: ['contentCategories'],
  },
};

type PrefetchKey = keyof typeof PREFETCH_CONFIG;

/**
 * Checks if the user is authenticated and has the required permissions.
 * Redirects to the homepage if checks fail.
 *
 * @param to - The route being navigated to.
 * @param from - The route being navigated from.
 * @param next - The navigation guard's next function.
 * @param requiredPermission - The specific permission string to check for (e.g., 'release:create').
 */
export function requirePermission(
  to: RouteLocationNormalized,
  from: RouteLocationNormalized,
  next: NavigationGuardNext,
  requiredPermission: string,
) {
  // 1. Synchronously get the account status from the cache.
  const accountStatus = queryClient.getQueryData<AccountStatusResponse>(['accountStatus']);

  // 2. Check for the permission.
  // We use `?.` (optional chaining) to safely access `permissions` in case accountStatus is undefined.
  const hasPermission = accountStatus?.permissions.includes(requiredPermission) ?? false;

  if (hasPermission) {
    // 3. If the user has permission, allow them to proceed.
    next();
  } else {
    // 4. If the user does not have permission, redirect to the homepage.
    console.warn(`Redirecting: User lacks required permission ('${requiredPermission}') for route '${to.path}'.`);
    next({ path: '/' });
  }
}

const routes: Array<RouteRecordRaw> = [
  {
    path: '/',
    component: HomePage,
  },
  {
    path: '/account',
    name: 'Account',
    component: AccountPage,

  },
  {
    path: '/upload',
    name: 'Upload',
    component: UploadPage,
    beforeEnter: (to, from, next) => {
      // We pass the specific permission required for this route
      requirePermission(to, from, next, 'release:create');
    },
  },
  {
    path: '/admin',
    name: 'Admin Website',
    component: AdminPage,
    beforeEnter: (to, from, next) => {
      // Example for a more complex check. You would create a dedicated helper for this.
      const accountStatus = queryClient.getQueryData<AccountStatusResponse>(['accountStatus']);
      const isAdmin = accountStatus?.isAdmin || accountStatus?.roles.includes('moderator') || false;
      if (isAdmin) {
        next();
      } else {
        next({ path: '/' });
      }
    },
  },
  {
    path: '/about',
    component: AboutPage,
  },
  {
    path: '/privacy-policy',
    component: PrivacyPolicyPage,
  },
  {
    path: '/terms',
    component: TermsPage,
  },
  {
    path: '/release/:id',
    name: 'Release',
    component: ReleasePage,
    props: true,
    beforeEnter: async (to, from, next) => {
      // 1. Get the release ID from the route params.
      const id = to.params.id as string;

      // Ensure we have an ID to work with.
      if (!id) {
        console.error('Release page navigation attempted without an ID.');
        // Optionally, redirect to a 404 page or the homepage.
        next({ path: '/' });
        return;
      }

      console.log(`Release Guard: Pre-fetching data for release ID: ${id}`);

      // 2. Define the specific query key for this release.
      const queryKey = ['release', id];

      // 3. Check if data for this specific release is already in the cache.
      // This is a crucial optimization. If the user clicks a release, then navigates
      // away and clicks the same release again, we don't need to re-fetch.
      if (queryClient.getQueryData(queryKey)) {
        console.log(`Cache hit for release ${id}. Skipping fetch.`);
        next();
        return;
      }

      try {
        // 4. Fetch the data from your fast API.
        const response = await fetch(`${API_URL}/releases/${id}`);

        if (response.ok) {
          const releaseData = await response.json();

          // 5. Seed the cache with the fetched data.
          queryClient.setQueryData(queryKey, releaseData);
          console.log(`✅ Cache seeded for release ${id}.`);
        } else {
          // Handle cases where the release is not found (404) or other server errors.
          console.error(`API Error fetching release ${id}: Status ${response.status}`);
          // You might want to clear any stale data if it exists and redirect.
          queryClient.setQueryData(queryKey, undefined);
          // Optionally redirect to a 'not-found' page.
          // For now, we'll just proceed and let the component handle the empty state.
        }
      } catch (error) {
        console.error(`Fetch failed for release ${id}:`, error);
      } finally {
        // 6. Always call next() to allow navigation to proceed.
        next();
      }
    },
  },
  {
    path: '/featured/:category',
    component: CategoryPage,
    props: route => ({ ...route.params, showAll: true }),
  },
  {
    path: '/series/:id',
    name: 'Series',
    component: SeriesPage,
    props: true,
  },
];

const router = createRouter({
  history: createWebHashHistory(),
  routes,
  scrollBehavior() {
    return { top: 0 };
  },
});

let hasAttemptedPrefetch = false;

router.beforeEach(async (to, from, next) => {
  if (hasAttemptedPrefetch) {
    next();
    return;
  }

  console.log('Global Guard: First-time app entry. Checking API health for pre-fetching...');
  hasAttemptedPrefetch = true; // Set this immediately to ensure this entire block runs only once.

  try {
    // 1. Perform a quick health check with a short timeout.
    // We use AbortController to enforce a timeout on the fetch call.
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), 1500); // 1.5-second timeout

    const healthResponse = await fetch(`${API_URL}/health`, { signal: controller.signal });
    clearTimeout(timeoutId); // Clear the timeout if the fetch completes in time

    if (!healthResponse.ok) {
      throw new Error(`API health check failed with status: ${healthResponse.status}`);
    }

    // 2. If health check passes, proceed with pre-fetching.
    console.log('API is healthy. Proceeding with pre-fetching and cache seeding...');
    const prefetchKeys = Object.keys(PREFETCH_CONFIG) as PrefetchKey[];
    const promises = prefetchKeys.map(key => fetch(PREFETCH_CONFIG[key].url));
    const results = await Promise.allSettled(promises);

    console.groupCollapsed('API Pre-fetch Responses & Cache Seeding');
    // Process each result
    results.forEach(async (result, index) => {
      const key = prefetchKeys[index];
      const config = PREFETCH_CONFIG[key];

      if (result.status === 'fulfilled' && result.value.ok) {
        let data = await result.value.json();
        
        // Transform the data to match what the query hooks return
        if (key === 'initialReleases' && Array.isArray(data)) {
          data = data.map((r: any) => ({
            ...r,
            metadata: r.metadata ? JSON.parse(r.metadata) : undefined,
          }));
        } else if (key === 'initialContentCategories' && Array.isArray(data)) {
          data = data.map((c: any) => ({
            ...c,
            metadataSchema: c.metadataSchema ? JSON.parse(c.metadataSchema) : undefined,
          }));
        }
        
        queryClient.setQueryData(config.queryKey, data);
        console.log(`✅ [SUCCESS & CACHE SEEDED] ${key}:`, data);
      } else if (result.status === 'fulfilled' && !result.value.ok) {
        console.error(`❌ [API ERROR] for ${key} at ${result.value.url}: Server responded with status ${result.value.status}`);
      } else if (result.status === 'rejected') {
        console.error(`❌ [FETCH FAILED] for ${key}:`, result.reason.message);
      }
    });

    console.groupEnd();

  } catch (error) {
    console.error('An unexpected error occurred during global data pre-fetching:', error);
  } finally {
    // ALWAYS call next() to allow the initial navigation to complete.
    next();
  }
});


export default router;
