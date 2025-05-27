import { describe, it, expect } from 'vitest';
import { formatTime } from './utils';

describe('formatTime', () => {
  it('should format milliseconds to HH:MM:SS or MM:SS', () => {
    expect(formatTime(0)).toBe('00:00');
    expect(formatTime(1000)).toBe('00:01'); // 1 second
    expect(formatTime(60000)).toBe('01:00'); // 1 minute
    expect(formatTime(3600000)).toBe('01:00:00'); // 1 hour
    expect(formatTime(3661000)).toBe('01:01:01'); // 1 hour, 1 minute, 1 second
    expect(formatTime(59000)).toBe('00:59');
    expect(formatTime(3599000)).toBe('59:59');
  });

  it('should handle NaN by returning 00:00', () => {
    expect(formatTime(NaN)).toBe('00:00');
  });

  it('should correctly format times less than 10 units with leading zeros', () => {
    expect(formatTime(5000)).toBe('00:05'); // 5 seconds
    expect(formatTime(5 * 60 * 1000)).toBe('05:00'); // 5 minutes
    expect(formatTime(5 * 60 * 60 * 1000)).toBe('05:00:00'); // 5 hours
  });
}); 