export function formatTime(ms: number): string {
  const seconds = Math.ceil(Math.max(0, Number.isFinite(ms) ? ms : 0) / 1000);
  return `${Math.floor(seconds / 60)
    .toString()
    .padStart(2, '0')}:${(seconds % 60).toString().padStart(2, '0')}`;
}
