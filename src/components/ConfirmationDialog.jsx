import { AlertTriangle, Check, RotateCcw } from "lucide-react";
import { useEffect, useState, useRef } from "react";

export function ConfirmationDialog({
  visible,
  message,
  timeoutSecs,
  onConfirm,
  onRollback,
}) {
  const [remaining, setRemaining] = useState(timeoutSecs ?? 10);
  const intervalRef = useRef(null);

  useEffect(() => {
    if (!visible) {
      setRemaining(timeoutSecs ?? 10);
      return;
    }
    setRemaining(timeoutSecs ?? 10);
    intervalRef.current = setInterval(() => {
      setRemaining((r) => {
        if (r <= 1) {
          clearInterval(intervalRef.current);
          onRollback?.();
          return 0;
        }
        return r - 1;
      });
    }, 1000);
    return () => clearInterval(intervalRef.current);
  }, [visible]);

  if (!visible) return null;

  return (
    <div className="fixed inset-0 z-9999 flex items-center justify-center bg-black/40 backdrop-blur-sm animate-fade-in">
      <div className="card w-full max-w-md border border-base-300 bg-base-200 shadow-2xl">
        <div className="card-body items-center gap-4 p-6 text-center">
          <div className="rounded-full bg-warning/20 p-4 text-warning">
            <AlertTriangle className="size-8" />
          </div>
          <div className="space-y-1">
            <h3 className="text-lg font-bold text-base-content">
              Layout Change
            </h3>
            <p className="text-sm text-base-content/70">{message}</p>
          </div>
          <div className="flex items-center gap-2 text-sm font-mono text-base-content/60">
            <span className="countdown font-mono text-xl text-warning">
              <span style={{ "--value": remaining }}>{remaining}</span>
            </span>
            <span>seconds remaining</span>
          </div>
          <div className="flex w-full gap-3 pt-2">
            <button
              className="btn btn-error btn-outline flex-1"
              onClick={onRollback}
            >
              <RotateCcw className="size-4" />
              Revert
            </button>
            <button
              className="btn btn-success flex-1 shadow-md shadow-success/20"
              onClick={onConfirm}
            >
              <Check className="size-4" />
              Keep Change
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
