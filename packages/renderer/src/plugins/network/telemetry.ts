// Network Telemetry
// Tracks network performance and request statistics

export interface NetworkRequest {
  adapter: string;
  method: string;
  success: boolean;
  duration: number;
  error?: string;
  timestamp: number;
}

export class NetworkTelemetry {
  private requests: NetworkRequest[] = [];
  private maxHistory = 1000;

  recordRequest(request: NetworkRequest): void {
    this.requests.push(request);
    if (this.requests.length > this.maxHistory) {
      this.requests.shift();
    }
  }

  getStats(): {
    totalRequests: number;
    successRate: number;
    avgResponseTime: number;
    byAdapter: Record<string, any>;
  } {
    if (this.requests.length === 0) {
      return {
        totalRequests: 0,
        successRate: 0,
        avgResponseTime: 0,
        byAdapter: {}
      };
    }

    const totalRequests = this.requests.length;
    const successfulRequests = this.requests.filter(r => r.success).length;
    const successRate = (successfulRequests / totalRequests) * 100;

    const totalDuration = this.requests.reduce((sum, r) => sum + r.duration, 0);
    const avgResponseTime = totalDuration / totalRequests;

    const byAdapter: Record<string, any> = {};
    this.requests.forEach(request => {
      if (!byAdapter[request.adapter]) {
        byAdapter[request.adapter] = {
          total: 0,
          success: 0,
          avgDuration: 0,
          errors: [] as string[]
        };
      }

      byAdapter[request.adapter].total++;
      if (request.success) {
        byAdapter[request.adapter].success++;
      } else if (request.error) {
        byAdapter[request.adapter].errors.push(request.error);
      }
    });

    // Calculate average duration by adapter
    Object.keys(byAdapter).forEach(adapter => {
      const adapterRequests = this.requests.filter(r => r.adapter === adapter);
      const totalDuration = adapterRequests.reduce((sum, r) => sum + r.duration, 0);
      byAdapter[adapter].avgDuration = totalDuration / adapterRequests.length;
    });

    return {
      totalRequests,
      successRate,
      avgResponseTime,
      byAdapter
    };
  }

  getRecentFailures(limit: number = 50): NetworkRequest[] {
    return this.requests.filter(r => !r.success).slice(-limit);
  }

  getRequestHistory(limit: number = 100): NetworkRequest[] {
    return [...this.requests].slice(-limit);
  }

  clearHistory(): void {
    this.requests = [];
  }
}