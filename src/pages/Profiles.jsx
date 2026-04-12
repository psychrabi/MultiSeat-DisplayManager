import { CheckIcon, PlusIcon } from "../components/Icons.jsx";
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

const ProfilesPage = () => {
  // ===== STORE =====
  const profiles = useProfileStore((s) => s.profiles);
  const selectedProfileUser = useProfileStore((s) => s.selectedProfileUser);
  const setSelectedProfileUser = useProfileStore((s) => s.setSelectedUser);

  const currentUser = useAppStore((s) => s.currentUser);

  // ===== DERIVED =====
  const users = Object.keys(profiles.users ?? {});
  const selectedProfile = selectedProfileUser
    ? profiles.users[selectedProfileUser]
    : null;

  // ===== UI =====
  return (
    <div className="page active">
      <div className="topbar">
        <span className="page-title">{PAGE_TITLES.profiles}</span>

        <div className="topbar-actions">
          <button className="btn primary">
            <PlusIcon />
            New Profile
          </button>
        </div>
      </div>

      <div className="scroll-area">
        <div className="profiles-layout">
          {/* USERS LIST */}
          <div className="profiles-list-panel">
            <div className="panel-header">
              <span className="panel-title">Users</span>
            </div>

            <div className="profile-list">
              {users.length === 0 ? (
                <div className="empty-copy">
                  No profiles yet.
                </div>
              ) : (
                users.map((username, index) => {
                  const count = Object.keys(
                    profiles.users[username]?.assignments ?? {}
                  ).length;

                  return (
                    <div
                      key={username}
                      className={`profile-item${
                        selectedProfileUser === username ? " active" : ""
                      }`}
                      onClick={() => setSelectedProfileUser(username)}
                    >
                      <div
                        className="profile-avatar"
                        style={{
                          background:
                            AVATAR_COLORS[index % AVATAR_COLORS.length],
                        }}
                      >
                        {getUserInitial(username)}
                      </div>

                      <div>
                        <div className="profile-item-name">
                          {getUserShortName(username)}
                        </div>

                        <div className="profile-item-count">
                          {count} monitor{count === 1 ? "" : "s"} assigned
                        </div>
                      </div>
                    </div>
                  );
                })
              )}
            </div>
          </div>

          {/* PROFILE DETAIL */}
          <div className="profile-detail">
            {!selectedProfileUser || !selectedProfile ? (
              <EmptyProfileState />
            ) : (
              <>
                <div className="panel-header">
                  <span className="panel-title">
                    Profile: {getUserShortName(selectedProfileUser)}
                  </span>

                  {selectedProfileUser === currentUser && (
                    <span className="badge active-badge">YOU</span>
                  )}
                </div>

                <div className="profile-detail-content">
                  {Object.entries(selectedProfile.assignments ?? {}).length === 0 ? (
                    <div className="empty-copy">
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
                          <div key={key} className="assignment-row">
                            <div>
                              <div className="assignment-device">
                                {getAssignmentMonitorName(key, assignment)}
                              </div>

                              <div className="assignment-mode">
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
    </div>
  );
};

export default ProfilesPage;