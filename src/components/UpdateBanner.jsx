import { memo } from "react";
import { Download, X } from "lucide-react";
import { useUpdater } from "../hooks/useUpdater";

const UpdateBanner = memo(() => {
  const {
    update,
    checking,
    downloading,
    progress,
    error,
    dismissUpdate,
    installUpdate,
  } = useUpdater();

  if (!checking && !update && !error) return null;

  if (checking) {
    return (
      <div className="alert alert-info shadow-sm rounded-none border-b border-info/20">
        <div className="flex items-center gap-2 text-sm">
          <span className="loading loading-spinner loading-xs"></span>
          Checking for updates...
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="alert alert-error shadow-sm rounded-none border-b border-error/20">
        <div className="flex w-full items-center justify-between gap-3">
          <span className="text-sm">Update check failed: {error}</span>
          <button className="btn btn-ghost btn-xs" onClick={dismissUpdate}>
            <X className="size-3" />
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className="alert alert-info shadow-lg rounded-none border-b border-info/20">
      <div className="flex w-full items-center justify-between gap-3">
        <div className="flex items-center gap-2">
          <Download className="size-4 shrink-0" />
          <span className="text-sm">
            {downloading
              ? `Downloading update ${update.version}... ${progress}%`
              : `Update ${update.version} is available`}
          </span>
          {downloading && (
            <progress
              className="progress progress-info w-32"
              value={progress}
              max="100"
            />
          )}
        </div>

        <div className="flex items-center gap-2 shrink-0">
          <button
            className="btn btn-primary btn-xs"
            onClick={installUpdate}
            disabled={downloading}
          >
            {downloading ? "Downloading..." : "Install"}
          </button>
          <button
            className="btn btn-ghost btn-xs"
            onClick={dismissUpdate}
            disabled={downloading}
          >
            <X className="size-3" />
          </button>
        </div>
      </div>

      {error && (
        <div className="text-xs text-error mt-1">
          Update failed: {error}
        </div>
      )}
    </div>
  );
});

export default UpdateBanner;
