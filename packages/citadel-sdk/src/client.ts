/**
 * Citadel SDK Client
 * HTTP-based client for the Citadel API
 */

import type {
  Release,
  FeaturedRelease,
  ContentCategory,
  Subscription,
  AccountStatusResponse,
  IdResponse,
  HashResponse,
  SearchOptions,
  AddInput,
  EditInput,
} from './types';

export interface CitadelClientConfig {
  baseUrl: string;
  publicKey?: string;
  signFn?: (message: string) => Promise<string>;
}

export interface ILensService {
  getRelease(id: string): Promise<Release | undefined>;
  getReleases(options?: SearchOptions): Promise<Release[]>;
  addRelease(data: AddInput): Promise<HashResponse>;
  editRelease(data: EditInput): Promise<IdResponse>;
  deleteRelease(id: string): Promise<IdResponse>;
  getFeaturedRelease(id: string): Promise<FeaturedRelease | undefined>;
  getFeaturedReleases(options?: SearchOptions): Promise<FeaturedRelease[]>;
  addFeaturedRelease(data: { releaseId: string; position?: number }): Promise<IdResponse>;
  deleteFeaturedRelease(id: string): Promise<IdResponse>;
  getContentCategories(options?: SearchOptions): Promise<ContentCategory[]>;
  addContentCategory(data: { name: string; slug: string; metadataSchema?: string }): Promise<IdResponse>;
  editContentCategory(data: { id: string; name?: string; slug?: string; metadataSchema?: string }): Promise<IdResponse>;
  deleteContentCategory(id: string): Promise<IdResponse>;
  getSubscriptions(options?: SearchOptions): Promise<Subscription[]>;
  getAccountStatus(publicKey: string): Promise<AccountStatusResponse>;
}

export class CitadelService implements ILensService {
  private baseUrl: string;
  private publicKey?: string;
  private signFn?: (message: string) => Promise<string>;
  peerbit?: {
    identity?: { publicKey: { toString(): string } };
    peerId?: { toString(): string };
  };
  siteProgram?: unknown;

  constructor(config: CitadelClientConfig | string = '/api/v1') {
    if (typeof config === 'string') {
      this.baseUrl = config;
    } else {
      this.baseUrl = config.baseUrl;
      this.publicKey = config.publicKey;
      this.signFn = config.signFn;
    }
  }

  setCredentials(publicKey: string, signFn: (message: string) => Promise<string>) {
    this.publicKey = publicKey;
    this.signFn = signFn;
  }

  private async getHeaders(body?: string, method?: string, path?: string): Promise<HeadersInit> {
    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
    };

    if (this.publicKey) {
      headers['X-Public-Key'] = this.publicKey;

      // Sign authenticated requests
      if (this.signFn && (method === 'POST' || method === 'PUT' || method === 'DELETE')) {
        const timestamp = Date.now().toString();
        let messageToSign: string;

        if (body) {
          messageToSign = `${timestamp}:${body}`;
        } else if (method === 'DELETE' && path) {
          messageToSign = `${timestamp}:DELETE:${path}`;
        } else {
          messageToSign = timestamp;
        }

        const signature = await this.signFn(messageToSign);
        headers['X-Signature'] = signature;
        headers['X-Timestamp'] = timestamp;
      }
    }

    return headers;
  }

  // Release methods
  async getRelease(id: string): Promise<Release | undefined> {
    const response = await fetch(`${this.baseUrl}/releases/${id}`, {
      headers: await this.getHeaders(),
    });

    if (!response.ok) {
      if (response.status === 404) return undefined;
      throw new Error(`Failed to get release: ${response.statusText}`);
    }

    return response.json();
  }

  async getReleases(options?: SearchOptions): Promise<Release[]> {
    const params = new URLSearchParams();
    if (options?.limit) params.set('limit', options.limit.toString());
    if (options?.offset) params.set('offset', options.offset.toString());
    if (options?.categoryId) params.set('categoryId', options.categoryId);
    if (options?.query) params.set('query', options.query);

    const url = params.toString()
      ? `${this.baseUrl}/releases?${params}`
      : `${this.baseUrl}/releases`;

    const response = await fetch(url, {
      headers: await this.getHeaders(),
    });

    if (!response.ok) {
      throw new Error(`Failed to get releases: ${response.statusText}`);
    }

    return response.json();
  }

  async addRelease(data: AddInput): Promise<HashResponse> {
    const body = JSON.stringify(data);
    const response = await fetch(`${this.baseUrl}/releases`, {
      method: 'POST',
      headers: await this.getHeaders(body, 'POST'),
      body,
    });

    if (!response.ok) {
      const error = await response.json().catch(() => ({ error: response.statusText }));
      throw new Error(error.error || `Failed to create release: ${response.statusText}`);
    }

    return response.json();
  }

  async editRelease(data: EditInput): Promise<IdResponse> {
    const { id, ...updateData } = data;
    const body = JSON.stringify(updateData);
    const response = await fetch(`${this.baseUrl}/releases/${id}`, {
      method: 'PUT',
      headers: await this.getHeaders(body, 'PUT'),
      body,
    });

    if (!response.ok) {
      const error = await response.json().catch(() => ({ error: response.statusText }));
      throw new Error(error.error || `Failed to update release: ${response.statusText}`);
    }

    return response.json();
  }

  async deleteRelease(id: string): Promise<IdResponse> {
    const path = `/releases/${id}`;
    const response = await fetch(`${this.baseUrl}${path}`, {
      method: 'DELETE',
      headers: await this.getHeaders(undefined, 'DELETE', path),
    });

    if (!response.ok) {
      const error = await response.json().catch(() => ({ error: response.statusText }));
      throw new Error(error.error || `Failed to delete release: ${response.statusText}`);
    }

    return response.json();
  }

  // Featured release methods
  async getFeaturedRelease(id: string): Promise<FeaturedRelease | undefined> {
    const response = await fetch(`${this.baseUrl}/featured-releases/${id}`, {
      headers: await this.getHeaders(),
    });

    if (!response.ok) {
      if (response.status === 404) return undefined;
      throw new Error(`Failed to get featured release: ${response.statusText}`);
    }

    return response.json();
  }

  async getFeaturedReleases(_options?: SearchOptions): Promise<FeaturedRelease[]> {
    const response = await fetch(`${this.baseUrl}/featured-releases`, {
      headers: await this.getHeaders(),
    });

    if (!response.ok) {
      throw new Error(`Failed to get featured releases: ${response.statusText}`);
    }

    return response.json();
  }

  async addFeaturedRelease(data: { releaseId: string; position?: number }): Promise<IdResponse> {
    const body = JSON.stringify(data);
    const response = await fetch(`${this.baseUrl}/featured-releases`, {
      method: 'POST',
      headers: await this.getHeaders(body, 'POST'),
      body,
    });

    if (!response.ok) {
      const error = await response.json().catch(() => ({ error: response.statusText }));
      throw new Error(error.error || `Failed to add featured release: ${response.statusText}`);
    }

    return response.json();
  }

  async deleteFeaturedRelease(id: string): Promise<IdResponse> {
    const path = `/featured-releases/${id}`;
    const response = await fetch(`${this.baseUrl}${path}`, {
      method: 'DELETE',
      headers: await this.getHeaders(undefined, 'DELETE', path),
    });

    if (!response.ok) {
      const error = await response.json().catch(() => ({ error: response.statusText }));
      throw new Error(error.error || `Failed to delete featured release: ${response.statusText}`);
    }

    return response.json();
  }

  // Content category methods
  async getContentCategories(_options?: SearchOptions): Promise<ContentCategory[]> {
    const response = await fetch(`${this.baseUrl}/content-categories`, {
      headers: await this.getHeaders(),
    });

    if (!response.ok) {
      throw new Error(`Failed to get content categories: ${response.statusText}`);
    }

    return response.json();
  }

  async addContentCategory(data: { name: string; slug: string; metadataSchema?: string }): Promise<IdResponse> {
    const body = JSON.stringify(data);
    const response = await fetch(`${this.baseUrl}/content-categories`, {
      method: 'POST',
      headers: await this.getHeaders(body, 'POST'),
      body,
    });

    if (!response.ok) {
      const error = await response.json().catch(() => ({ error: response.statusText }));
      throw new Error(error.error || `Failed to add content category: ${response.statusText}`);
    }

    return response.json();
  }

  async editContentCategory(data: { id: string; name?: string; slug?: string; metadataSchema?: string }): Promise<IdResponse> {
    const { id, ...updateData } = data;
    const body = JSON.stringify(updateData);
    const response = await fetch(`${this.baseUrl}/content-categories/${id}`, {
      method: 'PUT',
      headers: await this.getHeaders(body, 'PUT'),
      body,
    });

    if (!response.ok) {
      const error = await response.json().catch(() => ({ error: response.statusText }));
      throw new Error(error.error || `Failed to update content category: ${response.statusText}`);
    }

    return response.json();
  }

  async deleteContentCategory(id: string): Promise<IdResponse> {
    const path = `/content-categories/${id}`;
    const response = await fetch(`${this.baseUrl}${path}`, {
      method: 'DELETE',
      headers: await this.getHeaders(undefined, 'DELETE', path),
    });

    if (!response.ok) {
      const error = await response.json().catch(() => ({ error: response.statusText }));
      throw new Error(error.error || `Failed to delete content category: ${response.statusText}`);
    }

    return response.json();
  }

  // Subscription methods
  async getSubscriptions(_options?: SearchOptions): Promise<Subscription[]> {
    const response = await fetch(`${this.baseUrl}/subscriptions`, {
      headers: await this.getHeaders(),
    });

    if (!response.ok) {
      throw new Error(`Failed to get subscriptions: ${response.statusText}`);
    }

    return response.json();
  }

  // Account methods
  async getAccountStatus(publicKey: string): Promise<AccountStatusResponse> {
    const encodedKey = encodeURIComponent(publicKey);
    const response = await fetch(`${this.baseUrl}/account/${encodedKey}`, {
      headers: await this.getHeaders(),
    });

    if (!response.ok) {
      return { isAdmin: false, roles: [], permissions: [] };
    }

    return response.json();
  }

  // Admin methods
  async authorizeAdmin(publicKey: string): Promise<{ success: boolean; message: string }> {
    const body = JSON.stringify({ publicKey });
    const response = await fetch(`${this.baseUrl}/admin/authorize`, {
      method: 'POST',
      headers: await this.getHeaders(body, 'POST'),
      body,
    });

    if (!response.ok) {
      const error = await response.json().catch(() => ({ error: response.statusText }));
      throw new Error(error.error || `Failed to authorize admin: ${response.statusText}`);
    }

    return response.json();
  }

  // Bulk operations
  async bulkDeleteAllReleases(): Promise<{ success: boolean; deleted: number }> {
    const path = '/releases/bulk/delete-all';
    const response = await fetch(`${this.baseUrl}${path}`, {
      method: 'POST',
      headers: await this.getHeaders(undefined, 'POST', path),
    });

    if (!response.ok) {
      const error = await response.json().catch(() => ({ error: response.statusText }));
      throw new Error(error.error || `Failed to bulk delete releases: ${response.statusText}`);
    }

    return response.json();
  }
}

// Alias for backwards compatibility
export { CitadelService as LensService };
