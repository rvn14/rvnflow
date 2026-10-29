import { describe, it, expect } from 'vitest';
import { dotProgress, taskProgress } from './progress';
import { formatTime } from './time';
describe('timer rendering', () => {
  it('formats minute boundaries without negative or invalid time', () => {
    expect(formatTime(1500000)).toBe('25:00');
    expect(formatTime(59999)).toBe('01:00');
    expect(formatTime(1)).toBe('00:01');
    expect(formatTime(-1)).toBe('00:00');
    expect(formatTime(NaN)).toBe('00:00');
  });
  it('has no completed dots at zero and ten at completion', () => {
    expect(dotProgress(0, 100).completed).toBe(0);
    expect(dotProgress(100, 100).completed).toBe(10);
    expect(dotProgress(999, 100).completed).toBe(10);
  });
  it('handles fractional progress and invalid duration', () => {
    expect(dotProgress(55, 100)).toMatchObject({ completed: 5, fraction: 0.5 });
    for (const duration of [0, -1, NaN, Infinity])
      expect(dotProgress(5, duration).progress).toBe(0);
    expect(dotProgress(-20, 100).completed).toBe(0);
  });
  it('keeps task and timer completion separate', () => {
    expect(taskProgress([])).toEqual({ completed: 0, total: 0, percent: 0 });
    expect(
      taskProgress([{ completed: true }, { completed: false }]).percent,
    ).toBe(50);
  });
});
