/**
 * HTTP-based Lens Service
 * Calls the lens-v2-node REST API backend via UBTS transactions
 */

export interface Release {
  id: string;
  name: string;
  categoryId: string;
  categorySlug: string;
  contentCID: string;
  thumbnailCID?: string;
  metadata?: Record<string, unknown>;
  siteAddress: string;
  postedBy: string;
  createdAt: string;
}

export interface IdResponse {
  id: string;
  success: boolean;
}

export class HttpLensService {
  private baseUrl: string;
  private publicKey?: string;

  constructor(baseUrl: string = '/api/v1') {
    this.baseUrl = baseUrl;
  }

  setPublicKey(publicKey: string) {
    this.publicKey = publicKey;
  }

  private getHeaders(): HeadersInit {
    const headers: HeadersInit = {
      'Content-Type': 'application/json',
    };

    if (this.publicKey) {
      headers['X-Public-Key'] = this.publicKey;
    }

    return headers;
  }

  async getReleases(): Promise<Release[]> {
    const response = await fetch(`${this.baseUrl}/releases`, {
      headers: this.getHeaders(),
    });

    if (!response.ok) {
      throw new Error(`Failed to get releases: ${response.statusText}`);
    }

    return await response.json();
  }

  async getRelease(id: string): Promise<Release> {
    const response = await fetch(`${this.baseUrl}/releases/${id}`, {
      headers: this.getHeaders(),
    });

    if (!response.ok) {
      if (response.status === 404) {
        throw new Error('Release not found');
      }
      throw new Error(`Failed to get release: ${response.statusText}`);
    }

    return await response.json();
  }

  async deleteRelease(id: string): Promise<IdResponse> {
    const response = await fetch(`${this.baseUrl}/releases/${id}`, {
      method: 'DELETE',
      headers: this.getHeaders(),
    });

    if (!response.ok) {
      const error = await response.json().catch(() => ({ error: response.statusText }));
      throw new Error(error.error || 'Failed to delete release');
    }

    const result = await response.json();
    return {
      id,
      success: result.success || true,
    };
  }

  async createRelease(data: {
    name: string;
    categoryId: string;
    categorySlug: string;
    contentCID: string;
    thumbnailCID?: string;
    metadata?: Record<string, unknown>;
  }): Promise<IdResponse> {
    const response = await fetch(`${this.baseUrl}/releases`, {
      method: 'POST',
      headers: this.getHeaders(),
      body: JSON.stringify(data),
    });

    if (!response.ok) {
      const error = await response.json().catch(() => ({ error: response.statusText }));
      throw new Error(error.error || 'Failed to create release');
    }

    return await response.json();
  }

  async updateRelease(id: string, data: {
    name: string;
    categoryId: string;
    contentCID: string;
    thumbnailCID?: string;
    metadata?: Record<string, unknown>;
    siteAddress: string;
    postedBy: string;
  }): Promise<IdResponse> {
    const response = await fetch(`${this.baseUrl}/releases/${id}`, {
      method: 'PUT',
      headers: this.getHeaders(),
      body: JSON.stringify(data),
    });

    if (!response.ok) {
      const error = await response.json().catch(() => ({ error: response.statusText }));
      throw new Error(error.error || 'Failed to update release');
    }

    return await response.json();
  }

  // Featured releases management
  async getFeaturedReleases(): Promise<Release[]> {
    const response = await fetch(`${this.baseUrl}/featured-releases`, {
      headers: this.getHeaders(),
    });

    if (!response.ok) {
      throw new Error(`Failed to get featured releases: ${response.statusText}`);
    }

    return await response.json();
  }

  // Admin operations
  async authorizeAdmin(publicKey: string): Promise<{ success: boolean; message: string }> {
    const response = await fetch(`${this.baseUrl}/admin/authorize`, {
      method: 'POST',
      headers: this.getHeaders(),
      body: JSON.stringify({ publicKey }),
    });

    if (!response.ok) {
      throw new Error(`Failed to authorize admin: ${response.statusText}`);
    }

    return await response.json();
  }

  async getAccountStatus(publicKey: string): Promise<{
    isAdmin: boolean;
    roles: string[];
    permissions: string[];
  }> {
    const response = await fetch(`${this.baseUrl}/account/${publicKey}`, {
      headers: this.getHeaders(),
    });

    if (!response.ok) {
      throw new Error(`Failed to get account status: ${response.statusText}`);
    }

    return await response.json();
  }
}
