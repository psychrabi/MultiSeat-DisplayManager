import { UsersIcon } from "./Icons";

export function EmptyProfileState() {
  return (
    <div className="empty-state">
      <UsersIcon />
      <p>Select a user profile to view details</p>
    </div>
  );
}