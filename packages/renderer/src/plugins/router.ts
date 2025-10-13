import type { NavigationGuardNext, RouteLocationNormalized } from 'vue-router';
import { createRouter, createWebHashHistory, type RouteRecordRaw } from 'vue-router';
import { multiaddr } from '@multiformats/multiaddr';
// Keep HomePage as direct import since it's the landing page
import HomePage from '/@/views/homePage.vue';
import { queryClient } from './tanstackQuery';
import type { AccountStatusResponse } from '@riffcc/lens-sdk';
import { getApiUrl as getRuntimeApiUrl } from '/@/utils/runtimeConfig';

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
const ArtistPage = () => import('../views/artistPage.vue');
const AlbumPage = () => import('../views/albumPage.vue');
const PodcastPage = () => import('../views/podcastPage.vue');
const PodcastEpisodePage = () => import('../views/podcastEpisodePage.vue');
const AudiobookPage = () => import('../views/audiobookPage.vue');
const BookPage = () => import('../views/bookPage.vue');
const ReaderPage = () => import('../views/readerPage.vue');
const BooksPage = () => import('../views/booksPage.vue');

// Use runtime configuration for API URL (supports both build-time and runtime injection)
export const API_URL = getRuntimeApiUrl();
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
    beforeEnter: async (to, from, next) => {
      // Check admin status using new identity system
      try {
        // Import ed25519 library
        const ed25519 = await import('@noble/ed25519');

        // Try to get identity from localStorage
        const identitySeed = localStorage.getItem('lens_identity_seed');
        if (!identitySeed) {
          console.warn('[Router] No identity found, redirecting to home');
          next({ path: '/' });
          return;
        }

        // Derive public key using @noble/ed25519 (matches useIdentity.ts)
        const seedBytes = new Uint8Array(identitySeed.length / 2);
        for (let i = 0; i < identitySeed.length; i += 2) {
          seedBytes[i / 2] = parseInt(identitySeed.substring(i, i + 2), 16);
        }

        // Seed is the private key for ed25519
        const privateKey = seedBytes;
        const publicKeyBytes = await ed25519.getPublicKeyAsync(privateKey);
        const publicKeyHex = Array.from(publicKeyBytes).map(b => b.toString(16).padStart(2, '0')).join('');
        const publicKey = `ed25519p/${publicKeyHex}`;

        console.log('[Router] Checking admin status for:', publicKey);

        // Check authorization status
        const encodedKey = encodeURIComponent(publicKey);
        const response = await fetch(`${API_URL}/account/${encodedKey}`);

        if (response.ok) {
          const accountStatus = await response.json();
          const isAdmin = accountStatus?.isAdmin || accountStatus?.roles?.includes('moderator') || false;

          if (isAdmin) {
            console.log('[Router] Admin access granted for', publicKey);
            next();
          } else {
            console.warn('[Router] Not an admin, redirecting to home. Status:', accountStatus);
            next({ path: '/' });
          }
        } else {
          console.warn('[Router] Failed to check admin status:', response.status, response.statusText);
          next({ path: '/' });
        }
      } catch (error) {
        console.error('[Router] Error checking admin status:', error);
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
  // Simplified category routes
  {
    path: '/music',
    component: CategoryPage,
    props: () => ({ category: 'music', showAll: true }),
  },
  {
    path: '/movies',
    component: CategoryPage,
    props: () => ({ category: 'movies', showAll: true }),
  },
  {
    path: '/tv',
    component: CategoryPage,
    props: () => ({ category: 'tv-shows', showAll: true }),
  },
  {
    path: '/books',
    component: BooksPage,
  },
  {
    path: '/audiobooks',
    component: CategoryPage,
    props: () => ({ category: 'audiobooks', showAll: true }),
  },
  {
    path: '/games',
    component: CategoryPage,
    props: () => ({ category: 'games', showAll: true }),
  },
  // Legacy route for backwards compatibility
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
    beforeEnter: async (to, from, next) => {
      const id = to.params.id as string;

      if (!id) {
        console.error('Series page navigation attempted without an ID.');
        next({ path: '/' });
        return;
      }

      console.log(`Series Guard: Pre-fetching data for series ID: ${id}`);

      const queryKey = ['release', id];

      if (queryClient.getQueryData(queryKey)) {
        console.log(`Cache hit for series ${id}. Skipping fetch.`);
        next();
        return;
      }

      try {
        const response = await fetch(`${API_URL}/releases/${id}`);

        if (response.ok) {
          const seriesData = await response.json();
          queryClient.setQueryData(queryKey, seriesData);
          console.log(`✅ Cache seeded for series ${id}.`);
        } else {
          console.error(`API Error fetching series ${id}: Status ${response.status}`);
          queryClient.setQueryData(queryKey, undefined);
        }
      } catch (error) {
        console.error(`Fetch failed for series ${id}:`, error);
      } finally {
        next();
      }
    },
  },
  {
    path: '/artist/:id',
    name: 'Artist',
    component: ArtistPage,
    props: true,
    beforeEnter: async (to, from, next) => {
      const id = to.params.id as string;

      if (!id) {
        console.error('Artist page navigation attempted without an ID.');
        next({ path: '/' });
        return;
      }

      console.log(`Artist Guard: Pre-fetching data for artist ID: ${id}`);

      const queryKey = ['release', id];

      if (queryClient.getQueryData(queryKey)) {
        console.log(`Cache hit for artist ${id}. Skipping fetch.`);
        next();
        return;
      }

      try {
        const response = await fetch(`${API_URL}/releases/${id}`);

        if (response.ok) {
          const artistData = await response.json();
          queryClient.setQueryData(queryKey, artistData);
          console.log(`✅ Cache seeded for artist ${id}.`);
        } else {
          console.error(`API Error fetching artist ${id}: Status ${response.status}`);
          queryClient.setQueryData(queryKey, undefined);
        }
      } catch (error) {
        console.error(`Fetch failed for artist ${id}:`, error);
      } finally {
        next();
      }
    },
  },
  {
    path: '/album/:id',
    name: 'Album',
    component: AlbumPage,
    props: true,
    beforeEnter: async (to, from, next) => {
      const id = to.params.id as string;

      if (!id) {
        console.error('Album page navigation attempted without an ID.');
        next({ path: '/' });
        return;
      }

      console.log(`Album Guard: Pre-fetching data for album ID: ${id}`);

      const queryKey = ['release', id];

      if (queryClient.getQueryData(queryKey)) {
        console.log(`Cache hit for album ${id}. Skipping fetch.`);
        next();
        return;
      }

      try {
        const response = await fetch(`${API_URL}/releases/${id}`);

        if (response.ok) {
          const albumData = await response.json();
          queryClient.setQueryData(queryKey, albumData);
          console.log(`✅ Cache seeded for album ${id}.`);
        } else {
          console.error(`API Error fetching album ${id}: Status ${response.status}`);
          queryClient.setQueryData(queryKey, undefined);
        }
      } catch (error) {
        console.error(`Fetch failed for album ${id}:`, error);
      } finally {
        next();
      }
    },
  },
  {
    path: '/podcast/:id',
    name: 'Podcast',
    component: PodcastPage,
    props: true,
    beforeEnter: async (to, from, next) => {
      const id = to.params.id as string;

      if (!id) {
        console.error('Podcast page navigation attempted without an ID.');
        next({ path: '/' });
        return;
      }

      console.log(`Podcast Guard: Pre-fetching data for podcast ID: ${id}`);

      const queryKey = ['release', id];

      if (queryClient.getQueryData(queryKey)) {
        console.log(`Cache hit for podcast ${id}. Skipping fetch.`);
        next();
        return;
      }

      try {
        const response = await fetch(`${API_URL}/releases/${id}`);

        if (response.ok) {
          const podcastData = await response.json();
          queryClient.setQueryData(queryKey, podcastData);
          console.log(`✅ Cache seeded for podcast ${id}.`);
        } else {
          console.error(`API Error fetching podcast ${id}: Status ${response.status}`);
          queryClient.setQueryData(queryKey, undefined);
        }
      } catch (error) {
        console.error(`Fetch failed for podcast ${id}:`, error);
      } finally {
        next();
      }
    },
  },
  {
    path: '/podcast-episode/:id',
    name: 'PodcastEpisode',
    component: PodcastEpisodePage,
    props: true,
    beforeEnter: async (to, from, next) => {
      const id = to.params.id as string;

      if (!id) {
        console.error('Podcast episode page navigation attempted without an ID.');
        next({ path: '/' });
        return;
      }

      console.log(`Podcast Episode Guard: Pre-fetching data for episode ID: ${id}`);

      const queryKey = ['release', id];

      if (queryClient.getQueryData(queryKey)) {
        console.log(`Cache hit for podcast episode ${id}. Skipping fetch.`);
        next();
        return;
      }

      try {
        const response = await fetch(`${API_URL}/releases/${id}`);

        if (response.ok) {
          const episodeData = await response.json();
          queryClient.setQueryData(queryKey, episodeData);
          console.log(`✅ Cache seeded for podcast episode ${id}.`);
        } else {
          console.error(`API Error fetching podcast episode ${id}: Status ${response.status}`);
          queryClient.setQueryData(queryKey, undefined);
        }
      } catch (error) {
        console.error(`Fetch failed for podcast episode ${id}:`, error);
      } finally {
        next();
      }
    },
  },
  {
    path: '/audiobook/:id',
    name: 'Audiobook',
    component: AudiobookPage,
    props: true,
    beforeEnter: async (to, from, next) => {
      const id = to.params.id as string;

      if (!id) {
        console.error('Audiobook page navigation attempted without an ID.');
        next({ path: '/' });
        return;
      }

      console.log(`Audiobook Guard: Pre-fetching data for audiobook ID: ${id}`);

      const queryKey = ['release', id];

      if (queryClient.getQueryData(queryKey)) {
        console.log(`Cache hit for audiobook ${id}. Skipping fetch.`);
        next();
        return;
      }

      try {
        const response = await fetch(`${API_URL}/releases/${id}`);

        if (response.ok) {
          const audiobookData = await response.json();
          queryClient.setQueryData(queryKey, audiobookData);
          console.log(`✅ Cache seeded for audiobook ${id}.`);
        } else {
          console.error(`API Error fetching audiobook ${id}: Status ${response.status}`);
          queryClient.setQueryData(queryKey, undefined);
        }
      } catch (error) {
        console.error(`Fetch failed for audiobook ${id}:`, error);
      } finally {
        next();
      }
    },
  },
  {
    path: '/book/:id',
    name: 'Book',
    component: BookPage,
    props: true,
    beforeEnter: async (to, from, next) => {
      const id = to.params.id as string;

      if (!id) {
        console.error('Book page navigation attempted without an ID.');
        next({ path: '/' });
        return;
      }

      console.log(`Book Guard: Pre-fetching data for book ID: ${id}`);

      const queryKey = ['release', id];

      if (queryClient.getQueryData(queryKey)) {
        console.log(`Cache hit for book ${id}. Skipping fetch.`);
        next();
        return;
      }

      try {
        const response = await fetch(`${API_URL}/releases/${id}`);

        if (response.ok) {
          const bookData = await response.json();
          queryClient.setQueryData(queryKey, bookData);
          console.log(`✅ Cache seeded for book ${id}.`);
        } else {
          console.error(`API Error fetching book ${id}: Status ${response.status}`);
          queryClient.setQueryData(queryKey, undefined);
        }
      } catch (error) {
        console.error(`Fetch failed for book ${id}:`, error);
      } finally {
        next();
      }
    },
  },
  {
    path: '/read/:id',
    name: 'Reader',
    component: ReaderPage,
    props: true,
    beforeEnter: async (to, from, next) => {
      const id = to.params.id as string;

      if (!id) {
        console.error('Reader page navigation attempted without an ID.');
        next({ path: '/' });
        return;
      }

      console.log(`Reader Guard: Pre-fetching data for book ID: ${id}`);

      const queryKey = ['release', id];

      if (queryClient.getQueryData(queryKey)) {
        console.log(`Cache hit for reader ${id}. Skipping fetch.`);
        next();
        return;
      }

      try {
        const response = await fetch(`${API_URL}/releases/${id}`);

        if (response.ok) {
          const bookData = await response.json();
          queryClient.setQueryData(queryKey, bookData);
          console.log(`✅ Cache seeded for reader ${id}.`);
        } else {
          console.error(`API Error fetching book for reader ${id}: Status ${response.status}`);
          queryClient.setQueryData(queryKey, undefined);
        }
      } catch (error) {
        console.error(`Fetch failed for reader ${id}:`, error);
      } finally {
        next();
      }
    },
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
    // Process each result - use for...of to properly await async operations
    for (let index = 0; index < results.length; index++) {
      const result = results[index];
      const key = prefetchKeys[index];
      const config = PREFETCH_CONFIG[key];

      if (result.status === 'fulfilled' && result.value.ok) {
        let data = await result.value.json();

        // Transform the data to match what the query hooks return
        if (key === 'initialReleases' && Array.isArray(data)) {
          data = data.map((r: any) => ({
            ...r,
            // metadata is already an object from the API, no need to parse
            metadata: r.metadata,
          }));
        } else if (key === 'initialContentCategories' && Array.isArray(data)) {
          data = data.map((c: any) => ({
            ...c,
            // metadataSchema is already an object from the API, no need to parse
            metadataSchema: c.metadataSchema,
          }));
        }

        queryClient.setQueryData(config.queryKey, data);
        console.log(`✅ [SUCCESS & CACHE SEEDED] ${key}:`, data);
      } else if (result.status === 'fulfilled' && !result.value.ok) {
        console.error(`❌ [API ERROR] for ${key} at ${result.value.url}: Server responded with status ${result.value.status}`);
      } else if (result.status === 'rejected') {
        console.error(`❌ [FETCH FAILED] for ${key}:`, result.reason.message);
      }
    }

    console.groupEnd();

  } catch (error) {
    console.error('An unexpected error occurred during global data pre-fetching:', error);
  } finally {
    // ALWAYS call next() to allow the initial navigation to complete.
    next();
  }
});


export default router;
