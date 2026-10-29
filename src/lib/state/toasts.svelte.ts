export type ToastKind = 'success' | 'info' | 'warning' | 'error';
class ToastState {
  current = $state<{ id: number; kind: ToastKind; message: string } | null>(
    null,
  );
  private timer: ReturnType<typeof setTimeout> | undefined;
  show(message: string, kind: ToastKind = 'info') {
    clearTimeout(this.timer);
    this.current = { id: Date.now(), kind, message };
    if (kind !== 'error') this.timer = setTimeout(() => this.dismiss(), 4500);
  }
  dismiss() {
    clearTimeout(this.timer);
    this.current = null;
  }
}
export const toasts = new ToastState();
