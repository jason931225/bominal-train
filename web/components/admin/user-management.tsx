"use client";

import { useState, useEffect, useCallback } from "react";

import { useLocale } from "@/components/locale-provider";
import {
  UI_BUTTON_DANGER_SM,
  UI_BUTTON_OUTLINE_SM,
  UI_BUTTON_PRIMARY,
  UI_CARD_MD,
  UI_CHIP_BASE,
  UI_FIELD,
  UI_KICKER,
  UI_TITLE_MD,
} from "@/lib/ui";

type UserSummary = {
  id: string;
  email: string;
  display_name: string | null;
  role: string;
  created_at: string;
  last_seen_at: string | null;
  session_count: number;
  task_count: number;
};

type UserDetail = {
  id: string;
  email: string;
  display_name: string | null;
  phone_number: string | null;
  role: string;
  created_at: string;
  updated_at: string;
  email_verified_at: string | null;
  session_count: number;
  active_session_count: number;
  task_count: number;
  secret_count: number;
};

type UserListResponse = {
  users: UserSummary[];
  total: number;
  page: number;
  page_size: number;
};

export function UserManagement() {
  const { locale, t } = useLocale();
  const [users, setUsers] = useState<UserSummary[]>([]);
  const [total, setTotal] = useState(0);
  const [page, setPage] = useState(1);
  const [pageSize] = useState(10);
  const [search, setSearch] = useState("");
  const [loading, setLoading] = useState(true);
  const [selectedUser, setSelectedUser] = useState<UserDetail | null>(null);
  const [actionLoading, setActionLoading] = useState(false);
  const [message, setMessage] = useState<{ type: "success" | "error"; text: string } | null>(null);

  const fetchUsers = useCallback(async () => {
    setLoading(true);
    try {
      const params = new URLSearchParams({
        page: page.toString(),
        page_size: pageSize.toString(),
      });
      if (search) params.set("search", search);

      const res = await fetch(`/api/admin/users?${params}`, { credentials: "include" });
      if (res.ok) {
        const data: UserListResponse = await res.json();
        setUsers(data.users);
        setTotal(data.total);
      }
    } catch (e) {
      console.error("Failed to fetch users", e);
    } finally {
      setLoading(false);
    }
  }, [page, pageSize, search]);

  useEffect(() => {
    fetchUsers();
  }, [fetchUsers]);

  const fetchUserDetail = async (userId: string) => {
    try {
      const res = await fetch(`/api/admin/users/${userId}`, { credentials: "include" });
      if (res.ok) {
        const data: UserDetail = await res.json();
        setSelectedUser(data);
      }
    } catch (e) {
      console.error("Failed to fetch user detail", e);
    }
  };

  const updateRole = async (userId: string, newRole: "admin" | "user") => {
    setActionLoading(true);
    try {
      const res = await fetch(`/api/admin/users/${userId}/role`, {
        method: "PATCH",
        headers: { "Content-Type": "application/json" },
        credentials: "include",
        body: JSON.stringify({ role: newRole }),
      });
      if (res.ok) {
        const roleLabel = newRole === "admin" ? t("admin.users.roles.admin") : t("admin.users.roles.user");
        setMessage({ type: "success", text: t("admin.users.roleUpdated", { role: roleLabel }) });
        fetchUsers();
        if (selectedUser?.id === userId) {
          fetchUserDetail(userId);
        }
      } else {
        const data = await res.json();
        setMessage({ type: "error", text: data.detail || t("admin.users.failedUpdateRole") });
      }
    } finally {
      setActionLoading(false);
    }
  };

  const revokeSessions = async (userId: string) => {
    if (!confirm(t("admin.users.confirmRevokeSessions"))) return;
    setActionLoading(true);
    try {
      const res = await fetch(`/api/admin/users/${userId}/revoke-sessions`, {
        method: "POST",
        credentials: "include",
      });
      if (res.ok) {
        const data = await res.json();
        setMessage({ type: "success", text: data.message });
        if (selectedUser?.id === userId) {
          fetchUserDetail(userId);
        }
      } else {
        const data = await res.json();
        setMessage({ type: "error", text: data.detail || t("admin.users.failedRevokeSessions") });
      }
    } finally {
      setActionLoading(false);
    }
  };

  const deleteUser = async (userId: string, email: string) => {
    if (!confirm(t("admin.users.confirmDeleteUser1", { email }))) return;
    if (!confirm(t("admin.users.confirmDeleteUser2"))) return;
    setActionLoading(true);
    try {
      const res = await fetch(`/api/admin/users/${userId}`, {
        method: "DELETE",
        credentials: "include",
      });
      if (res.ok) {
        setMessage({ type: "success", text: t("admin.users.userDeleted", { email }) });
        setSelectedUser(null);
        fetchUsers();
      } else {
        const data = await res.json();
        setMessage({ type: "error", text: data.detail || t("admin.users.failedDeleteUser") });
      }
    } finally {
      setActionLoading(false);
    }
  };

  const totalPages = Math.ceil(total / pageSize);

  const formatDate = (dateStr: string | null) => {
    if (!dateStr) return t("admin.users.never");
    return new Date(dateStr).toLocaleString(locale);
  };

  const roleChipClass = (role: string) => {
    if (role === "admin") {
      return `${UI_CHIP_BASE} bg-rose-100 text-rose-700 border border-rose-200`;
    }
    return `${UI_CHIP_BASE} bg-slate-100 text-slate-600 border border-slate-200`;
  };

  const roleLabel = (role: string) => {
    if (role === "admin") return t("admin.users.roles.admin");
    if (role === "user") return t("admin.users.roles.user");
    return role;
  };

  return (
    <section className={UI_CARD_MD}>
      <p className={UI_KICKER}>{t("admin.users.kicker")}</p>
      <h2 className={`mt-2 ${UI_TITLE_MD}`}>{t("admin.users.title")}</h2>

      {/* Message */}
      {message && (
        <div
          className={`mt-4 rounded-xl px-4 py-3 text-sm ${
            message.type === "success"
              ? "bg-green-50 text-green-700 border border-green-200"
              : "bg-rose-50 text-rose-700 border border-rose-200"
          }`}
        >
          {message.text}
          <button
            onClick={() => setMessage(null)}
            className="ml-2 font-medium hover:underline"
          >
            {t("admin.users.dismiss")}
          </button>
        </div>
      )}

      {/* Search */}
      <div className="mt-4">
        <input
          type="text"
          placeholder={t("admin.users.searchPlaceholder")}
          className={UI_FIELD}
          value={search}
          onChange={(e) => {
            setSearch(e.target.value);
            setPage(1);
          }}
        />
      </div>

      {/* User List */}
      <div className="mt-4 overflow-hidden rounded-xl border border-blossom-100">
        <table className="w-full text-sm">
          <thead className="bg-blossom-50/50 text-left text-xs uppercase tracking-wide text-slate-500">
            <tr>
              <th className="px-4 py-3">{t("admin.users.table.user")}</th>
              <th className="px-4 py-3">{t("admin.users.table.role")}</th>
              <th className="px-4 py-3 hidden sm:table-cell">{t("admin.users.table.sessions")}</th>
              <th className="px-4 py-3 hidden md:table-cell">{t("admin.users.table.lastSeen")}</th>
              <th className="px-4 py-3">{t("admin.users.table.actions")}</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-blossom-50">
            {loading ? (
              <tr>
                <td colSpan={5} className="px-4 py-8 text-center text-slate-400">
                  {t("admin.users.loading")}
                </td>
              </tr>
            ) : users.length === 0 ? (
              <tr>
                <td colSpan={5} className="px-4 py-8 text-center text-slate-400">
                  {t("admin.users.noUsersFound")}
                </td>
              </tr>
            ) : (
              users.map((user) => (
                <tr key={user.id} className="hover:bg-blossom-50/30">
                  <td className="px-4 py-3">
                    <div className="font-medium text-slate-700">{user.email}</div>
                    {user.display_name && (
                      <div className="text-xs text-slate-400">{user.display_name}</div>
                    )}
                  </td>
                  <td className="px-4 py-3">
                    <span className={roleChipClass(user.role)}>{roleLabel(user.role)}</span>
                  </td>
                  <td className="px-4 py-3 hidden sm:table-cell text-slate-500">
                    {user.session_count}
                  </td>
                  <td className="px-4 py-3 hidden md:table-cell text-slate-500 text-xs">
                    {formatDate(user.last_seen_at)}
                  </td>
                  <td className="px-4 py-3">
                    <button
                      onClick={() => fetchUserDetail(user.id)}
                      className={UI_BUTTON_OUTLINE_SM}
                    >
                      {t("admin.users.actions.view")}
                    </button>
                  </td>
                </tr>
              ))
            )}
          </tbody>
        </table>
      </div>

      {/* Pagination */}
      {totalPages > 1 && (
        <div className="mt-4 flex items-center justify-between">
          <p className="text-sm text-slate-500">
            {t("admin.users.pagination.showing", {
              from: (page - 1) * pageSize + 1,
              to: Math.min(page * pageSize, total),
              total,
            })}
          </p>
          <div className="flex gap-2">
            <button
              onClick={() => setPage((p) => Math.max(1, p - 1))}
              disabled={page === 1}
              className={UI_BUTTON_OUTLINE_SM}
            >
              {t("admin.users.pagination.previous")}
            </button>
            <button
              onClick={() => setPage((p) => Math.min(totalPages, p + 1))}
              disabled={page === totalPages}
              className={UI_BUTTON_OUTLINE_SM}
            >
              {t("admin.users.pagination.next")}
            </button>
          </div>
        </div>
      )}

      {/* User Detail Modal */}
      {selectedUser && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40 p-4">
          <div className="w-full max-w-lg rounded-2xl bg-white p-6 shadow-xl">
            <div className="flex items-start justify-between">
              <div>
                <p className={UI_KICKER}>{t("admin.users.detail.kicker")}</p>
                <h3 className="mt-1 text-xl font-semibold text-slate-800">
                  {selectedUser.email}
                </h3>
              </div>
              <button
                onClick={() => setSelectedUser(null)}
                className="rounded-full p-1 text-slate-400 hover:bg-slate-100 hover:text-slate-600"
              >
                <svg className="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
            </div>

            <dl className="mt-4 grid grid-cols-2 gap-3 text-sm">
              <div>
                <dt className="text-slate-400">{t("auth.displayName")}</dt>
                <dd className="text-slate-700">{selectedUser.display_name || t("common.notSet")}</dd>
              </div>
              <div>
                <dt className="text-slate-400">{t("settings.phone")}</dt>
                <dd className="text-slate-700">{selectedUser.phone_number || t("common.notSet")}</dd>
              </div>
              <div>
                <dt className="text-slate-400">{t("admin.users.table.role")}</dt>
                <dd>
                  <span className={roleChipClass(selectedUser.role)}>{roleLabel(selectedUser.role)}</span>
                </dd>
              </div>
              <div>
                <dt className="text-slate-400">{t("admin.users.detail.emailVerified")}</dt>
                <dd className="text-slate-700">
                  {selectedUser.email_verified_at ? formatDate(selectedUser.email_verified_at) : t("admin.users.detail.notVerified")}
                </dd>
              </div>
              <div>
                <dt className="text-slate-400">{t("admin.users.detail.activeSessions")}</dt>
                <dd className="text-slate-700">
                  {selectedUser.active_session_count} / {selectedUser.session_count}
                </dd>
              </div>
              <div>
                <dt className="text-slate-400">{t("admin.users.detail.tasks")}</dt>
                <dd className="text-slate-700">{selectedUser.task_count}</dd>
              </div>
              <div>
                <dt className="text-slate-400">{t("admin.users.detail.secrets")}</dt>
                <dd className="text-slate-700">{selectedUser.secret_count}</dd>
              </div>
              <div>
                <dt className="text-slate-400">{t("common.created")}</dt>
                <dd className="text-slate-700 text-xs">{formatDate(selectedUser.created_at)}</dd>
              </div>
            </dl>

            {/* Actions */}
            <div className="mt-6 space-y-3 border-t border-slate-100 pt-4">
              <p className="text-xs font-medium uppercase tracking-wide text-slate-400">{t("admin.users.table.actions")}</p>

              <div className="flex flex-wrap gap-2">
                {selectedUser.role === "user" ? (
                  <button
                    onClick={() => updateRole(selectedUser.id, "admin")}
                    disabled={actionLoading}
                    className={UI_BUTTON_OUTLINE_SM}
                  >
                    {t("admin.users.actions.promoteToAdmin")}
                  </button>
                ) : (
                  <button
                    onClick={() => updateRole(selectedUser.id, "user")}
                    disabled={actionLoading}
                    className={UI_BUTTON_OUTLINE_SM}
                  >
                    {t("admin.users.actions.demoteToUser")}
                  </button>
                )}

                <button
                  onClick={() => revokeSessions(selectedUser.id)}
                  disabled={actionLoading || selectedUser.active_session_count === 0}
                  className={UI_BUTTON_OUTLINE_SM}
                >
                  {t("admin.users.actions.revokeSessions", { count: selectedUser.active_session_count })}
                </button>
              </div>

              <div className="pt-2">
                <button
                  onClick={() => deleteUser(selectedUser.id, selectedUser.email)}
                  disabled={actionLoading}
                  className={UI_BUTTON_DANGER_SM}
                >
                  {t("admin.users.actions.deleteUserPermanently")}
                </button>
              </div>
            </div>
          </div>
        </div>
      )}
    </section>
  );
}
