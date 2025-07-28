import { QueryClient } from '@tanstack/vue-query';

// Configure QueryClient with optimized defaults for P2P network
export const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      // Network timeout for queries - PeerBit needs time to find nodes
      networkMode: 'online',
      // Retry configuration
      retry: (failureCount, error) => {
        // Handle PeerBit-specific errors
        if (error?.message?.includes('delivery acknowledges from all nodes')) {
          return failureCount < 2; // Limited retries for connectivity
        }
        if (error?.message?.includes('Failed to get message')) {
          return failureCount < 3; // More retries for message delivery
        }
        return failureCount < 2; // Default retry
      },
      retryDelay: attemptIndex => Math.min(500 * Math.pow(2, attemptIndex), 2000),
      // Cache configuration
      staleTime: 1000 * 60, // 1 minute default
      gcTime: 1000 * 60 * 15, // 15 minutes
    },
    mutations: {
      retry: 1,
      retryDelay: 1000,
    },
  },
});
