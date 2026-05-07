import { useEffect } from "react";
import { useAppStore } from "../stores/useAppStore";

export function ToastContainer() {
  const toasts = useAppStore((s) => s.toasts);
  const dismissToast = useAppStore((s) => s.dismissToast);

  useEffect(() => {
    if (toasts.length === 0) return;

    const timers = toasts.map((t) =>
      setTimeout(() => dismissToast(t.id), 3000)
    );

    return () => timers.forEach(clearTimeout);
  }, [toasts, dismissToast]);

  if (toasts.length === 0) return null;

  return (
    <div className="toast toast-end toast-bottom z-[1000] pointer-events-none">
      {toasts.map((t) => {
        const alertType =
          t.type === "error" ? "alert-error"
          : t.type === "success" ? "alert-success"
          : t.type === "warn" ? "alert-warning"
          : "alert-info";

        return (
          <div
            key={t.id}
            className={`alert ${alertType} pointer-events-auto shadow-lg animate-fade-in`}
          >
            <span>{t.message}</span>
          </div>
        );
      })}
    </div>
  );
}
