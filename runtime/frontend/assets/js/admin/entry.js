import { errorMessage, requestJson } from "./common/api.js";
import { createConfirmModal } from "./common/modal.js";
import { escapeHtml } from "./common/utils.js";
import { renderAuditSection } from "./sections/audit.js";
import { renderConfigSection } from "./sections/config.js";
import { renderObservabilitySection } from "./sections/observability.js";
import { renderRuntimeSection } from "./sections/runtime.js";
import { renderSecuritySection } from "./sections/security.js";
import { renderUsersSection } from "./sections/users.js";

(() => {
  const shell = document.querySelector("[data-admin-section]");
  if (!shell) return;

  const section = shell.getAttribute("data-admin-section") || "";
  const content = document.getElementById("admin-content");
  const flash = document.getElementById("admin-flash");
  if (!content || !flash) return;

  const confirmModal = createConfirmModal();
  const cleanupCallbacks = [];
  const registerCleanup = (callback) => {
    if (typeof callback === "function") cleanupCallbacks.push(callback);
  };

  const runCleanup = () => {
    while (cleanupCallbacks.length) {
      const callback = cleanupCallbacks.pop();
      try {
        callback?.();
      } catch (_error) {
        // best effort cleanup
      }
    }
    confirmModal.teardown();
  };

  const setFlash = (kind, message) => {
    if (!message) {
      flash.textContent = "";
      flash.className = "mt-3 hidden";
      return;
    }
    flash.className = kind === "error" ? "mt-3 error-card" : "mt-3 empty-card";
    flash.textContent = message;
  };

  const ctx = {
    content,
    requestJson,
    errorMessage,
    setFlash,
    openConfirmModal: confirmModal.open,
    registerCleanup,
  };

  const renderers = {
    users: renderUsersSection,
    runtime: renderRuntimeSection,
    observability: renderObservabilitySection,
    security: renderSecuritySection,
    config: renderConfigSection,
    audit: renderAuditSection,
  };

  const renderer = renderers[section];
  if (!renderer) {
    content.innerHTML = '<div class="error-card">Unsupported admin section.</div>';
    return;
  }

  window.addEventListener("beforeunload", runCleanup, { once: true });
  renderer(ctx).catch((error) => {
    content.innerHTML = `<div class="error-card">${escapeHtml(String(error))}</div>`;
  });
})();
