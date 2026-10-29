export function dotProgress(elapsed: number, duration: number, count = 10) {
  const progress =
    Number.isFinite(elapsed) && Number.isFinite(duration) && duration > 0
      ? Math.min(1, Math.max(0, elapsed / duration))
      : 0;
  const total = Number.isFinite(count) ? Math.max(1, Math.floor(count)) : 10;
  return {
    progress,
    completed: Math.floor(progress * total),
    fraction: (progress * total) % 1,
    total,
  };
}
export function taskProgress(tasks: { completed: boolean }[]) {
  const completed = tasks.filter((t) => t.completed).length;
  return {
    completed,
    total: tasks.length,
    percent: tasks.length ? Math.round((completed / tasks.length) * 100) : 0,
  };
}
