// Client-side caching for federation index data
interface CacheEntry<T> {
  data: T;
  timestamp: number;
  ttl: number;
}

class FederationCache {
  private cache = new Map<string, CacheEntry<any>>();
  private readonly DEFAULT_TTL = 30000; // 30 seconds

  set<T>(key: string, data: T, ttl = this.DEFAULT_TTL): void {
    this.cache.set(key, {
      data,
      timestamp: Date.now(),
      ttl
    });

    // Send to service worker if available
    if ('serviceWorker' in navigator && navigator.serviceWorker.controller) {
      navigator.serviceWorker.controller.postMessage({
        type: 'CACHE_FEDERATION_DATA',
        url: `/federation/${key}`,
        data
      });
    }
  }

  get<T>(key: string): T | null {
    const entry = this.cache.get(key);
    if (!entry) return null;

    const age = Date.now() - entry.timestamp;
    if (age > entry.ttl) {
      this.cache.delete(key);
      return null;
    }

    console.log(`[FederationCache] Cache hit for ${key} (age: ${age}ms)`);
    return entry.data;
  }

  clear(): void {
    this.cache.clear();
    
    // Clear service worker cache too
    if ('serviceWorker' in navigator && navigator.serviceWorker.controller) {
      navigator.serviceWorker.controller.postMessage({
        type: 'CLEAR_CACHE'
      });
    }
  }

  // Pre-warm cache with data
  prewarm(key: string, dataPromise: Promise<any>): void {
    dataPromise.then(data => {
      this.set(key, data);
      console.log(`[FederationCache] Pre-warmed cache for ${key}`);
    }).catch(err => {
      console.error(`[FederationCache] Failed to pre-warm ${key}:`, err);
    });
  }
}

export const federationCache = new FederationCache();