import { Monitor, Plus, Target, Trash2, UserCheck, Users } from "lucide-react";
import { invoke } from "../api";
import { EmptyProfileState } from "../components/profiles/ui.jsx";
import {
  AVATAR_COLORS,
  getAssignmentMonitorName,
  getUserInitial,
  getUserShortName,
  PAGE_TITLES,
} from "../js/utils.jsx";
import { useAppStore } from "../stores/useAppStore";
import { useDisplayStore } from "../stores/useDisplayStore";
import { useProfileStore } from "../stores/useProfileStore";

const ProfilesPage = () => {
  const profiles = useProfileStore((s) => s.profiles);
  const selectedProfileUser = useProfileStore((s) => s.selectedProfileUser);
  const setSelectedProfileUser = useProfileStore((s) => s.setSelectedUser);
  const newProfileOpen = useProfileStore((s) => s.newProfileOpen);
  const newProfileUsername = useProfileStore((s) => s.newProfileUsername);
  const setNewProfileOpen = useProfileStore((s) => s.setNewProfileOpen);
  const setNewProfileUsername = useProfileStore((s) => s.setNewProfileUsername);
  const refreshProfiles = useProfileStore((s) => s.refreshProfiles);

  const refreshDisplays = useDisplayStore((s) => s.refreshDisplays);

  const currentUser = useAppStore((s) => s.currentUser);
  const activeProfile = useAppStore((s) => s.activeProfile);
  const setActiveProfile = useAppStore((s) => s.setActiveProfile);
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
  const allUsers =
    currentUser && !users.includes(currentUser)
      ? [currentUser, ...users]
      : users;

  const resolvedUser = selectedProfileUser ?? (currentUser || allUsers[0]);
  const savedProfile = resolvedUser ? profiles.users[resolvedUser] : null;
  const selectedProfile = savedProfile ?? { assignments: {} };

  return (
    <div className="">
      <div className="border border-base-300 bg-base-200/60 p-5 shadow-sm">
        <div className="flex flex-col gap-4 lg:flex-row lg:items-center lg:justify-between">
          <div className="space-y-1">
            <h2 className="text-2xl font-semibold text-base-content">
              {PAGE_TITLES.profiles}
            </h2>
            <p className="text-sm text-base-content/60">
              Manage user-specific display assignments and profiles.
            </p>
          </div>
          <div className="flex gap-3">
            <div
              className="tooltip tooltip-bottom"
              data-tip="Create a new profile"
            >
              <button
                className="btn btn-primary"
                onClick={() => setNewProfileOpen(true)}
              >
                <Plus className="size-4" />
                New Profile
              </button>
            </div>
          </div>
        </div>
      </div>

      <div className="grid gap-4 xl:grid-cols-[minmax(0,0.5fr)_minmax(300px,1fr)] p-4">
        <div className="card bg-base-200 border border-base-300 shadow-sm flex flex-col min-h-70 overflow-hidden">
          <div className="px-5 py-3.5 border-b border-base-300 flex items-center justify-between">
            <span className="text-xs font-bold uppercase tracking-widest text-base-content/60">
              Users
            </span>
            <span className="badge badge-ghost badge-sm">
              {allUsers.length}
            </span>
          </div>

          <div className="flex-1 overflow-y-auto">
            {allUsers.length === 0 ? (
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
                {allUsers.map((username, index) => {
                  const saved = profiles.users[username];
                  const count = Object.keys(saved?.assignments ?? {}).length;

                  return (
                    <li key={username}>
                      <div
                        className={`group flex items-center gap-3 p-3.5 transition-colors cursor-pointer ${
                          resolvedUser === username
                            ? "bg-primary/10 hover:bg-primary/15"
                            : "hover:bg-base-300/50"
                        }`}
                        onClick={() => setSelectedProfileUser(username)}
                      >
                        <div
                          className={`w-9 h-9 rounded-xl shrink-0 grid place-items-center text-xs font-bold shadow-sm bg-${AVATAR_COLORS[index % AVATAR_COLORS.length]}`}
                        >
                          {getUserInitial(username)}
                        </div>

                        <div className="flex flex-col min-w-0 flex-1">
                          <div
                            className={`text-sm font-medium truncate ${
                              resolvedUser === username
                                ? "text-primary"
                                : "text-base-content"
                            }`}
                          >
                            {getUserShortName(username)}
                          </div>
                          <div className="text-[10px] text-base-content/50 flex items-center gap-1.5">
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

                        {username !== activeProfile && (
                          <button
                            className="btn btn-ghost btn-xs btn-square opacity-0 group-hover:opacity-100 hover:opacity-100 transition-opacity"
                            data-tip="Apply & set as active"
                            onClick={async (e) => {
                              e.stopPropagation();
                              setActiveProfile(username);
                              setSelectedProfileUser(username);
                              if (profiles.users[username]) {
                                try {
                                  await invoke("apply_profile_for_user", {
                                    username,
                                  });
                                } catch (err) {
                                  console.error("Apply failed:", err);
                                }
                              }
                              await refreshProfiles();
                              await refreshDisplays();
                              pushToast(
                                `${getUserShortName(username)} set as active profile`,
                                "success",
                              );
                            }}
                            title="Apply and set as active profile"
                          >
                            <Target className="size-3" />
                          </button>
                        )}

                        {saved && (
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
                        )}
                      </div>
                    </li>
                  );
                })}
              </ul>
            )}
          </div>
        </div>

        <div className="card bg-base-200 border border-base-300 shadow-sm flex flex-col min-h-105 overflow-hidden">
          {!resolvedUser ? (
            <EmptyProfileState />
          ) : (
            <>
              <div className="px-5 py-3.5 border-b border-base-300 flex items-center justify-between">
                <div className="flex items-center gap-3">
                  <div className="text-xs font-bold uppercase tracking-widest text-base-content/60">
                    Profile: {getUserShortName(resolvedUser)}
                  </div>
                  {resolvedUser === currentUser && (
                    <span className="badge badge-success badge-outline badge-sm font-mono tracking-widest gap-1">
                      <UserCheck className="size-2.5" />
                      YOU
                    </span>
                  )}
                  {resolvedUser === activeProfile && (
                    <span className="badge badge-primary badge-outline badge-sm font-mono tracking-widest gap-1">
                      <Target className="size-2.5" />
                      ACTIVE
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
                      {savedProfile
                        ? "No monitor assignments"
                        : "Profile not saved yet"}
                    </p>
                    <p className="text-xs text-base-content/40">
                      {savedProfile
                        ? "Apply display settings from the Monitors page to create assignments."
                        : "Apply display settings and enable auto-save to create this profile."}
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
                            className="p-4 px-5 flex items-center gap-4 hover:bg-base-300/30 transition-colors "
                          >
                            <div className="rounded-lg bg-primary p-2.5 text-primary-content">
                              <Monitor className="size-5" />
                            </div>
                            <div className="min-w-0 flex-1">
                              <div className="text-xs font-mono text-base-content/70 mb-0.5 truncate">
                                {getAssignmentMonitorName(key, assignment)}
                              </div>
                              <div className="text-sm font-semibold text-primary">
                                {width}x{height} @{" "}
                                {assignment.mode.refresh_rate}
                                Hz
                              </div>
                              <div className="text-[10px] font-mono text-base-content/40">
                                {assignment.orientation ?? "landscape"} &middot;
                                {assignment.scale_factor ?? 100}% scale
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

      <dialog className={`modal ${newProfileOpen ? "modal-open" : ""}`}>
        <div className="modal-box bg-base-200 border border-base-300 shadow-2xl">
          <h3 className="font-bold text-lg mb-1">Create New Profile</h3>
          <p className="text-sm text-base-content/60 mb-5">
            Enter a username or profile name. The profile will be selected
            automatically after creation &mdash; apply monitor settings to save
            to it.
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
