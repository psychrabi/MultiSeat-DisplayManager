import { PlusIcon } from "../components/Icons.jsx";
import { EmptyProfileState } from "../components/ui.jsx";
import {
  AVATAR_COLORS,
  getAssignmentMonitorName,
  getUserInitial,
  getUserShortName,
  PAGE_TITLES,
} from "../js/utils.jsx";

import { useProfileStore } from "../stores/useProfileStore";
import { useAppStore } from "../stores/useAppStore";
import { Plus } from "lucide-react";
import { invoke } from "../api";

const ProfilesPage = () => {
  // ===== STORE =====
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

  // ===== ACTIONS =====
  const handleCreateProfile = async (e) => {
    e?.preventDefault();
    if (!newProfileUsername.trim()) return;

    try {
      await invoke("save_user_profile", {
        username: newProfileUsername,
        assignments: {}
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

  // ===== DERIVED =====
  const users = Object.keys(profiles.users ?? {});
  const selectedProfile = selectedProfileUser
    ? profiles.users[selectedProfileUser]
    : null;

  // ===== UI =====
  return (
    <div className="flex flex-col h-full active">
      <div className="flex items-center justify-between mb-4 pb-2 border-b border-base-300">
        <span className="text-xl font-semibold">{PAGE_TITLES.profiles}</span>

        <div className="flex gap-2">
          <button className="btn btn-primary btn-sm" onClick={() => setNewProfileOpen(true)}>
            <Plus />
            New Profile
          </button>
        </div>
      </div>

      <div className="flex-1 overflow-auto py-2">
        <div className="grid grid-cols-1 md:grid-cols-[280px_1fr] gap-6 h-full items-start">
          {/* USERS LIST */}
          <div className="card bg-base-200 border border-base-300 shadow-sm flex flex-col min-h-[420px] overflow-hidden">
            <div className="px-4 py-3 border-b border-base-300 flex items-center justify-between">
              <span className="text-xs font-bold uppercase tracking-widest text-base-content/60">Users</span>
            </div>

            <div className="flex-1 overflow-y-auto">
              {users.length === 0 ? (
                <div className="flex flex-col items-center justify-center gap-3 text-base-content/50 text-center p-6 h-full">
                  No profiles yet.
                </div>
              ) : (
                <ul className="menu bg-base-200 w-full p-0">
                  {users.map((username, index) => {
                    const count = Object.keys(
                      profiles.users[username]?.assignments ?? {}
                    ).length;

                    return (
                      <li key={username}>
                        <a
                          className={`flex items-center gap-3 p-3 rounded-none border-b border-base-300/50 hover:bg-base-300/50 transition-colors ${selectedProfileUser === username ? "bg-primary/10 hover:bg-primary/20" : ""
                            }`}
                          onClick={() => setSelectedProfileUser(username)}
                        >
                          <div
                            className={`w-8 h-8 rounded-full flex-shrink-0 grid place-items-center text-xs font-bold text-black ${AVATAR_COLORS[index % AVATAR_COLORS.length]}`}
                          >
                            {getUserInitial(username)}
                          </div>

                          <div className="flex flex-col">
                            <div className={`text-sm font-medium ${selectedProfileUser === username ? "text-primary" : "text-base-content"}`}>
                              {getUserShortName(username)}
                            </div>

                            <div className="text-[10px] font-mono text-base-content/50">
                              {count} monitor{count === 1 ? "" : "s"} assigned
                            </div>
                          </div>
                        </a>
                      </li>
                    );
                  })}
                </ul>
              )}
            </div>
          </div>

          {/* PROFILE DETAIL */}
          <div className="card bg-base-200 border border-base-300 shadow-sm flex flex-col min-h-[420px] overflow-hidden">
            {!selectedProfileUser || !selectedProfile ? (
              <EmptyProfileState />
            ) : (
              <>
                <div className="px-4 py-3 border-b border-base-300 flex items-center justify-between">
                  <span className="text-xs font-bold uppercase tracking-widest text-base-content/60">
                    Profile: {getUserShortName(selectedProfileUser)}
                  </span>

                  {selectedProfileUser === currentUser && (
                    <span className="badge badge-success badge-outline badge-sm font-mono tracking-widest">YOU</span>
                  )}
                </div>

                <div className="flex-1 overflow-y-auto">
                  {Object.entries(selectedProfile.assignments ?? {}).length === 0 ? (
                    <div className="flex flex-col items-center justify-center gap-3 text-base-content/50 text-center p-6 h-full">
                      No monitor assignments yet.
                    </div>
                  ) : (
                    Object.entries(selectedProfile.assignments ?? {}).map(
                      ([key, assignment]) => {
                        const width =
                          assignment.mode?.width ?? assignment.width ?? 0;
                        const height =
                          assignment.mode?.height ?? assignment.height ?? 0;

                        return (
                          <div key={key} className="p-4 px-5 border-b border-base-300 grid grid-cols-[1fr_auto] gap-4 items-center">
                            <div>
                              <div className="text-xs font-mono text-base-content/70 mb-1">
                                {getAssignmentMonitorName(key, assignment)}
                              </div>

                              <div className="text-sm font-semibold text-primary">
                                {width}x{height}
                              </div>
                            </div>
                          </div>
                        );
                      }
                    )
                  )}
                </div>
              </>
            )}
          </div>
        </div>
      </div>

      {/* NEW PROFILE MODAL */}
      <dialog className={`modal ${newProfileOpen ? "modal-open" : ""}`}>
        <div className="modal-box bg-base-200 border border-base-300">
          <h3 className="font-bold text-lg mb-4">Create New Profile</h3>
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
              onKeyDown={(e) => e.key === "Enter" && handleCreateProfile(e)}
            />
          </div>
          <div className="modal-action">
            <button className="btn" onClick={() => { setNewProfileOpen(false); setNewProfileUsername(""); }}>Cancel</button>
            <button className="btn btn-primary" onClick={handleCreateProfile} disabled={!newProfileUsername.trim()}>Create</button>
          </div>
        </div>
        <div className="modal-backdrop bg-base-300/60" onClick={() => { setNewProfileOpen(false); setNewProfileUsername(""); }}></div>
      </dialog>
    </div>
  );
};

export default ProfilesPage;