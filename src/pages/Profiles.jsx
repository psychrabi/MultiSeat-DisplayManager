import { EmptyProfileState } from "../components/profiles/ui.jsx";
import {
  AVATAR_COLORS,
  getAssignmentMonitorName,
  getUserInitial,
  getUserShortName,
  PAGE_TITLES,
} from "../js/utils.jsx";

import { useProfileStore } from "../stores/useProfileStore";
import { useAppStore } from "../stores/useAppStore";
import { Plus, Trash2, Monitor, UserCheck, Users } from "lucide-react";
import { invoke } from "../api";

const ProfilesPage = () => {
  const profiles = useProfileStore((s) => s.profiles);
  const selectedProfileUser = useProfileStore((s) => s.selectedProfileUser);
  const setSelectedProfileUser = useProfileStore((s) => s.setSelectedUser);
  const newProfileOpen = useProfileStore((s) => s.newProfileOpen);
  const newProfileUsername = useProfileStore((s) => s.newProfileUsername);
  const setNewProfileOpen = useProfileStore((s) => s.setNewProfileOpen);
  const setNewProfileUsername = useProfileStore((s) => s.setNewProfileUsername);
  const refreshProfiles = useProfileStore((s) => s.refreshProfiles);

  const currentUser = useAppStore((s) => s.currentUser);
  const pushToast = useAppStore((s) => s.pushToast);

  const handleCreateProfile = async (e) => {
    e?.preventDefault();
    if (!newProfileUsername.trim()) return;

    try {
      await invoke("save_user_profile", {
        username: newProfileUsername,
        assignments: {},
      });
      await refreshProfiles();
      setNewProfileOpen(false);
      setSelectedProfileUser(newProfileUsername);
      setNewProfileUsername("");
      pushToast("Profile created successfully", "success");
    } catch (err) {
      pushToast(`Error creating profile: ${err}`, "error");
    }
  };

  const handleDeleteProfile = async (username) => {
    try {
      await invoke("delete_user_profile", { username });
      await refreshProfiles();
      if (selectedProfileUser === username) {
        setSelectedProfileUser(null);
      }
      pushToast("Profile deleted", "success");
    } catch (err) {
      pushToast(`Error deleting profile: ${err}`, "error");
    }
  };

  const users = Object.keys(profiles.users ?? {});
  const selectedProfile = selectedProfileUser
    ? profiles.users[selectedProfileUser]
    : null;

  return (
    <div className="flex h-full flex-col">
      <div className="rounded-2xl border border-base-300 bg-base-200/60 p-5 shadow-sm mb-6">
        <div className="flex items-center justify-between">
          <div className="space-y-1">
            <h2 className="text-2xl font-semibold text-base-content">
              {PAGE_TITLES.profiles}
            </h2>
            <p className="text-sm text-base-content/60">
              Manage user-specific display assignments and profiles.
            </p>
          </div>
          <button
            className="btn btn-primary"
            onClick={() => setNewProfileOpen(true)}
          >
            <Plus className="size-4" />
            New Profile
          </button>
        </div>
      </div>

      <div className="flex-1 overflow-auto">
        <div className="grid grid-cols-1 md:grid-cols-[300px_1fr] gap-6 h-full items-start">
          <div className="card bg-base-200 border border-base-300 shadow-sm flex flex-col min-h-70 overflow-hidden">
            <div className="px-5 py-3.5 border-b border-base-300 flex items-center justify-between">
              <span className="text-xs font-bold uppercase tracking-widest text-base-content/60">
                Users
              </span>
              <span className="badge badge-ghost badge-sm">{users.length}</span>
            </div>

            <div className="flex-1 overflow-y-auto">
              {users.length === 0 ? (
                <div className="flex flex-col items-center justify-center gap-3 text-base-content/50 text-center p-8 h-full">
                  <div className="rounded-full bg-base-300 p-4">
                    <Users className="size-8 text-base-content/40" />
                  </div>
                  <p className="text-sm font-medium">No profiles yet</p>
                  <p className="text-xs text-base-content/40">
                    Create a profile to get started.
                  </p>
                </div>
              ) : (
                <ul className="menu bg-base-200 w-full p-0 divide-y divide-base-300/50">
                  {users.map((username, index) => {
                    const count = Object.keys(
                      profiles.users[username]?.assignments ?? {},
                    ).length;

                    return (
                      <li key={username}>
                        <div
                          className={`group flex items-center gap-3 p-3.5 transition-colors cursor-pointer ${
                            selectedProfileUser === username
                              ? "bg-primary/10 hover:bg-primary/15"
                              : "hover:bg-base-300/50"
                          }`}
                          onClick={() => setSelectedProfileUser(username)}
                        >
                          <div
                            className={`w-9 h-9 rounded-xl shrink-0 grid place-items-center text-xs font-bold text-black shadow-sm ${
                              AVATAR_COLORS[index % AVATAR_COLORS.length]
                            }`}
                          >
                            {getUserInitial(username)}
                          </div>

                          <div className="flex flex-col min-w-0 flex-1">
                            <div
                              className={`text-sm font-medium truncate ${
                                selectedProfileUser === username
                                  ? "text-primary"
                                  : "text-base-content"
                              }`}
                            >
                              {getUserShortName(username)}
                            </div>
                            <div className="text-[10px] font-mono text-base-content/50 flex items-center gap-1.5">
                              <Monitor className="size-3" />
                              {count} monitor{count === 1 ? "" : "s"}
                            </div>
                          </div>

                          {username === currentUser && (
                            <div className="badge badge-success badge-outline badge-sm font-mono tracking-widest gap-1">
                              <UserCheck className="size-2.5" />
                              YOU
                            </div>
                          )}

                          <button
                            className="btn btn-ghost btn-xs btn-square opacity-0 group-hover:opacity-100 hover:opacity-100 transition-opacity text-error"
                            onClick={(e) => {
                              e.stopPropagation();
                              handleDeleteProfile(username);
                            }}
                            title={`Delete ${getUserShortName(username)}`}
                          >
                            <Trash2 className="size-3.5" />
                          </button>
                        </div>
                      </li>
                    );
                  })}
                </ul>
              )}
            </div>
          </div>

          <div className="card bg-base-200 border border-base-300 shadow-sm flex flex-col min-h-105 overflow-hidden">
            {!selectedProfileUser || !selectedProfile ? (
              <EmptyProfileState />
            ) : (
              <>
                <div className="px-5 py-3.5 border-b border-base-300 flex items-center justify-between">
                  <div className="flex items-center gap-3">
                    <div className="text-xs font-bold uppercase tracking-widest text-base-content/60">
                      Profile: {getUserShortName(selectedProfileUser)}
                    </div>
                    {selectedProfileUser === currentUser && (
                      <span className="badge badge-success badge-outline badge-sm font-mono tracking-widest gap-1">
                        <UserCheck className="size-2.5" />
                        YOU
                      </span>
                    )}
                  </div>
                </div>

                <div className="flex-1 overflow-y-auto">
                  {Object.entries(selectedProfile.assignments ?? {}).length ===
                  0 ? (
                    <div className="flex flex-col items-center justify-center gap-3 text-base-content/50 text-center p-8 h-full">
                      <div className="rounded-full bg-base-300 p-4">
                        <Monitor className="size-8 text-base-content/40" />
                      </div>
                      <p className="text-sm font-medium">
                        No monitor assignments
                      </p>
                      <p className="text-xs text-base-content/40">
                        Assign monitors from the Monitors page.
                      </p>
                    </div>
                  ) : (
                    <div className="divide-y divide-base-300/50">
                      {Object.entries(selectedProfile.assignments ?? {}).map(
                        ([key, assignment]) => {
                          const width =
                            assignment.mode?.width ?? assignment.width ?? 0;
                          const height =
                            assignment.mode?.height ?? assignment.height ?? 0;

                          return (
                            <div
                              key={key}
                              className="p-4 px-5 flex items-center gap-4 hover:bg-base-300/30 transition-colors"
                            >
                              <div className="rounded-lg bg-primary/10 p-2.5 text-primary">
                                <Monitor className="size-5" />
                              </div>
                              <div className="min-w-0 flex-1">
                                <div className="text-xs font-mono text-base-content/70 mb-0.5 truncate">
                                  {getAssignmentMonitorName(key, assignment)}
                                </div>
                                <div className="text-sm font-semibold text-primary">
                                  {width}x{height} @{" "}
                                  {assignment.mode.refresh_rate}Hz
                                </div>
                                <div className="text-[10px] font-mono text-base-content/40 mt-0.5">
                                  {assignment.orientation ?? "landscape"}{" "}
                                  &middot; {assignment.scale_factor ?? 100}%
                                  scale
                                </div>
                              </div>
                            </div>
                          );
                        },
                      )}
                    </div>
                  )}
                </div>
              </>
            )}
          </div>
        </div>
      </div>

      <dialog className={`modal ${newProfileOpen ? "modal-open" : ""}`}>
        <div className="modal-box bg-base-200 border border-base-300 shadow-2xl">
          <h3 className="font-bold text-lg mb-1">Create New Profile</h3>
          <p className="text-sm text-base-content/60 mb-5">
            Enter a username or profile name for the new display assignment set.
          </p>
          <form onSubmit={handleCreateProfile}>
            <div className="form-control w-full">
              <label className="label">
                <span className="label-text">Profile Name</span>
              </label>
              <input
                type="text"
                placeholder="e.g. John Doe, Gaming, Work..."
                className="input input-bordered w-full bg-base-100"
                value={newProfileUsername}
                onChange={(e) => setNewProfileUsername(e.target.value)}
                autoFocus
              />
            </div>
            <div className="modal-action">
              <button
                type="button"
                className="btn btn-ghost"
                onClick={() => {
                  setNewProfileOpen(false);
                  setNewProfileUsername("");
                }}
              >
                Cancel
              </button>
              <button
                type="submit"
                className="btn btn-primary"
                disabled={!newProfileUsername.trim()}
              >
                Create
              </button>
            </div>
          </form>
        </div>
        <form method="dialog" className="modal-backdrop bg-base-300/60">
          <button
            onClick={() => {
              setNewProfileOpen(false);
              setNewProfileUsername("");
            }}
          >
            close
          </button>
        </form>
      </dialog>
    </div>
  );
};

export default ProfilesPage;
