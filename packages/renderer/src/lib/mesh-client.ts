/**
 * Mesh WebSocket Client
 *
 * Connects to Citadel mesh visualization backend and streams real-time
 * topology updates.
 */

import type { MeshUpdate } from '/@/types/mesh';

/**
 * Mesh client configuration
 */
export interface MeshClientConfig {
  /** WebSocket URL */
  url: string;
  /** Reconnect automatically on disconnect */
  autoReconnect: boolean;
  /** Reconnect delay in milliseconds */
  reconnectDelay: number;
  /** Maximum reconnect attempts (0 = infinite) */
  maxReconnectAttempts: number;
}

/**
 * Default configuration
 */
const DEFAULT_CONFIG: MeshClientConfig = {
  url: 'ws://localhost:5000/api/v1/mesh/stream',
  autoReconnect: true,
  reconnectDelay: 5000,
  maxReconnectAttempts: 0,
};

/**
 * Connection state
 */
export enum ConnectionState {
  DISCONNECTED = 'disconnected',
  CONNECTING = 'connecting',
  CONNECTED = 'connected',
  RECONNECTING = 'reconnecting',
  ERROR = 'error',
}

/**
 * Event handlers
 */
export interface MeshClientEventHandlers {
  onUpdate?: (update: MeshUpdate) => void;
  onConnect?: () => void;
  onDisconnect?: () => void;
  onError?: (error: Error) => void;
  onStateChange?: (state: ConnectionState) => void;
}

/**
 * Mesh WebSocket client
 */
export class MeshClient {
  private config: MeshClientConfig;
  private ws: WebSocket | null = null;
  private reconnectTimeout: ReturnType<typeof setTimeout> | null = null;
  private reconnectAttempts: number = 0;
  private intentionallyClosed: boolean = false;
  private connectionState: ConnectionState = ConnectionState.DISCONNECTED;
  private eventHandlers: MeshClientEventHandlers = {};

  constructor(config: Partial<MeshClientConfig> = {}) {
    this.config = { ...DEFAULT_CONFIG, ...config };
  }

  /**
   * Get current connection state
   */
  getState(): ConnectionState {
    return this.connectionState;
  }

  /**
   * Check if connected
   */
  isConnected(): boolean {
    return this.connectionState === ConnectionState.CONNECTED;
  }

  /**
   * Set event handlers
   */
  on(handlers: MeshClientEventHandlers): void {
    this.eventHandlers = { ...this.eventHandlers, ...handlers };
  }

  /**
   * Update connection state and notify listeners
   */
  private setState(state: ConnectionState): void {
    if (this.connectionState !== state) {
      this.connectionState = state;
      this.eventHandlers.onStateChange?.(state);
    }
  }

  /**
   * Connect to mesh stream
   */
  connect(url?: string): void {
    if (url) {
      this.config.url = url;
    }

    if (this.ws && this.ws.readyState === WebSocket.OPEN) {
      console.log('[MeshClient] Already connected');
      return;
    }

    this.intentionallyClosed = false;
    this.setState(ConnectionState.CONNECTING);

    console.log('[MeshClient] Connecting to:', this.config.url);

    try {
      this.ws = new WebSocket(this.config.url);

      this.ws.onopen = () => {
        console.log('[MeshClient] Connected');
        this.setState(ConnectionState.CONNECTED);
        this.reconnectAttempts = 0;
        this.eventHandlers.onConnect?.();
      };

      this.ws.onmessage = (event) => {
        try {
          const data = JSON.parse(event.data);

          // Handle mesh update messages
          if (this.isMeshUpdate(data)) {
            this.eventHandlers.onUpdate?.(data);
          } else {
            console.warn('[MeshClient] Unknown message type:', data);
          }
        } catch (error) {
          console.error('[MeshClient] Failed to parse message:', error);
          this.eventHandlers.onError?.(error as Error);
        }
      };

      this.ws.onerror = (event) => {
        console.error('[MeshClient] WebSocket error:', event);
        this.setState(ConnectionState.ERROR);
        this.eventHandlers.onError?.(new Error('WebSocket error'));
      };

      this.ws.onclose = () => {
        console.log('[MeshClient] Connection closed');
        this.ws = null;

        if (!this.intentionallyClosed && this.config.autoReconnect) {
          this.scheduleReconnect();
        } else {
          this.setState(ConnectionState.DISCONNECTED);
          this.eventHandlers.onDisconnect?.();
        }
      };
    } catch (error) {
      console.error('[MeshClient] Failed to create WebSocket:', error);
      this.setState(ConnectionState.ERROR);
      this.eventHandlers.onError?.(error as Error);

      if (this.config.autoReconnect) {
        this.scheduleReconnect();
      }
    }
  }

  /**
   * Schedule reconnection attempt
   */
  private scheduleReconnect(): void {
    if (this.reconnectTimeout) {
      return; // Already scheduled
    }

    // Check if we've exceeded max attempts
    if (
      this.config.maxReconnectAttempts > 0 &&
      this.reconnectAttempts >= this.config.maxReconnectAttempts
    ) {
      console.error('[MeshClient] Max reconnect attempts reached');
      this.setState(ConnectionState.DISCONNECTED);
      this.eventHandlers.onDisconnect?.();
      return;
    }

    this.reconnectAttempts++;
    this.setState(ConnectionState.RECONNECTING);

    console.log(
      `[MeshClient] Reconnecting in ${this.config.reconnectDelay}ms (attempt ${this.reconnectAttempts})...`,
    );

    this.reconnectTimeout = setTimeout(() => {
      this.reconnectTimeout = null;
      this.connect();
    }, this.config.reconnectDelay);
  }

  /**
   * Disconnect from mesh stream
   */
  disconnect(): void {
    this.intentionallyClosed = true;

    // Clear reconnect timeout
    if (this.reconnectTimeout) {
      clearTimeout(this.reconnectTimeout);
      this.reconnectTimeout = null;
    }

    // Close WebSocket
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }

    this.setState(ConnectionState.DISCONNECTED);
    this.eventHandlers.onDisconnect?.();
  }

  /**
   * Send a message to the server
   */
  send(data: any): void {
    if (!this.ws || this.ws.readyState !== WebSocket.OPEN) {
      console.warn('[MeshClient] Cannot send, not connected');
      return;
    }

    try {
      this.ws.send(JSON.stringify(data));
    } catch (error) {
      console.error('[MeshClient] Failed to send message:', error);
      this.eventHandlers.onError?.(error as Error);
    }
  }

  /**
   * Type guard for MeshUpdate
   */
  private isMeshUpdate(data: any): data is MeshUpdate {
    return (
      data &&
      typeof data.node_count === 'number' &&
      typeof data.connection_count === 'number' &&
      typeof data.avg_latency === 'number' &&
      Array.isArray(data.nodes) &&
      Array.isArray(data.connections)
    );
  }

  /**
   * Clean up resources
   */
  destroy(): void {
    this.disconnect();
    this.eventHandlers = {};
  }
}

/**
 * Create a mesh client with event handlers
 */
export function createMeshClient(
  config: Partial<MeshClientConfig> = {},
  handlers: MeshClientEventHandlers = {},
): MeshClient {
  const client = new MeshClient(config);
  client.on(handlers);
  return client;
}
