import { UsersIcon } from "./Icons";

export function EmptyProfileState() {
  return (
    <div className="flex flex-col items-center justify-center gap-3 text-base-content/50 text-center p-6 h-full min-h-[300px]">
      <div className="opacity-30 scale-150 mb-2">
        <UsersIcon />
      </div>
      <p className="text-sm">Select a user profile to view details</p>
    </div>
  );
}