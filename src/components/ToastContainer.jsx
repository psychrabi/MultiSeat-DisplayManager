import { useAppStore } from "../stores/useAppStore";

export function ToastContainer() {
  const toasts = useAppStore((s) => s.toasts);

  if (toasts.length === 0) return null;

  return (
    <div className="toast toast-end toast-bottom z-[1000] z-index-top pointer-events-none">
      {toasts.map((t) => (
        <div key={t.id} className={`alert alert-${t.type || "info"} pointer-events-auto shadow-lg`}>
          <span>{t.message}</span>
        </div>
      ))}
    </div>
  );
}