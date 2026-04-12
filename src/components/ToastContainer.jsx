import { useAppStore } from "../stores/useAppStore";

export function ToastContainer() {
  const toasts = useAppStore((s) => s.toasts);

return (
  <div id="toast-area">
    {toasts.map((t) => (
      <div key={t.id} className={`toast ${t.type}`}>
        {t.message}
      </div>
    ))}
  </div>
);
}