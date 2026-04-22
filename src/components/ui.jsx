import { Users } from "lucide-react";

export function EmptyProfileState() {
  return (
    <div className="flex h-full min-h-[300px] flex-col items-center justify-center gap-3 p-6 text-center text-base-content/50">
      <div className="rounded-full bg-base-300 p-4 text-base-content/40">
        <Users className="size-8" />
      </div>
      <p className="text-sm">Select a user profile to view details</p>
    </div>
  );
}
