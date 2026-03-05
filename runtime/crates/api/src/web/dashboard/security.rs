use bominal_ui::{DashboardSection, render_dashboard_bottom_nav};

use super::super::{
    app_shell_topbar, dashboard_desktop_sidebar, dashboard_settings_tabs, html_escape,
};
pub fn render_dashboard_settings(email: &str) -> String {
    let topbar = app_shell_topbar("Settings", &format!("Signed in as {}", html_escape(email)));
    let sidebar = dashboard_desktop_sidebar("security");
    let settings_tabs = dashboard_settings_tabs("account");
    let bottom_nav = render_dashboard_bottom_nav(DashboardSection::Settings);
    format!(
        r#"{topbar}
<main class="mx-auto w-full max-w-[480px] px-4 pb-24 pt-4 md:max-w-7xl md:px-6 md:pt-8">
  <div class="md:grid md:grid-cols-[220px_minmax(0,1fr)] md:items-start md:gap-4">
    {sidebar}
    <section class="space-y-4">
      {settings_tabs}
      <div class="space-y-4">
        <section class="summary-card p-4">
          <div class="flex items-center justify-between gap-2">
            <h3 class="text-sm font-semibold txt-strong">Passkeys</h3>
            <button
              id="passkey-create"
              type="button"
              class="btn-chip inline-flex h-9 w-9 items-center justify-center rounded-xl p-0"
              aria-label="Create passkey"
              title="Create passkey"
            >
              <img class="h-4 w-4 icon-muted" src="/assets/icons/runtime-ui/icon-plus-btn-chip-light.svgz" data-svgz-light="/assets/icons/runtime-ui/icon-plus-btn-chip-light.svgz" data-svgz-dark="/assets/icons/runtime-ui/icon-plus-btn-chip-dark.svgz" alt="" aria-hidden="true" />
            </button>
          </div>
          <div id="passkey-status" class="mt-3 hidden"></div>
          <div id="passkeys-list" class="mt-4 space-y-2"><div class="loading-card">Loading passkeys...</div></div>
        </section>

        <section class="summary-card p-4">
          <h3 class="text-sm font-semibold txt-strong">Change password</h3>
          <p class="mt-1 text-sm txt-supporting">Use upper/lowercase letters, numbers, and symbols.</p>
          <form id="password-change-form" class="mt-4 space-y-3">
            <label class="field-label" for="new-password">New password</label>
            <input id="new-password" type="password" autocomplete="new-password" class="field-input h-12 w-full" />
            <label class="field-label" for="confirm-password">Confirm new password</label>
            <input id="confirm-password" type="password" autocomplete="new-password" class="field-input h-12 w-full" />
            <div class="space-y-1">
              <div class="password-strength-track">
                <div id="password-strength-fill" class="password-strength-fill"></div>
              </div>
              <p id="password-strength-text" class="text-xs txt-supporting">Strength: weak</p>
              <p id="password-match-hint" class="text-xs txt-supporting">Passwords must match.</p>
            </div>
            <div class="action-group" data-action-group="single">
              <button
                type="submit"
                class="btn-primary h-12 w-full"
                data-action-role="primary"
              >
                Update password
              </button>
            </div>
          </form>
          <div id="password-change-status" class="mt-3 hidden"></div>
        </section>
      </div>

      <div id="security-modal" class="app-modal-backdrop hidden" aria-hidden="true">
        <div class="app-modal-card" role="dialog" aria-modal="true" aria-labelledby="security-modal-title">
          <h4 id="security-modal-title" class="text-base font-semibold txt-strong"></h4>
          <p id="security-modal-message" class="mt-2 text-sm txt-supporting"></p>
          <div id="security-modal-input-wrap" class="mt-3 hidden">
            <label id="security-modal-input-label" class="field-label" for="security-modal-input">Current password</label>
            <input id="security-modal-input" type="password" autocomplete="current-password" class="field-input h-12 w-full" />
          </div>
          <div class="action-pair mt-4" data-action-group="pair">
            <button
              id="security-modal-cancel"
              type="button"
              class="btn-ghost h-11 w-full"
              data-action-role="secondary"
            >
              Cancel
            </button>
            <button
              id="security-modal-confirm"
              type="button"
              class="btn-primary h-11 w-full"
              data-action-role="primary"
            >
              Confirm
            </button>
          </div>
        </div>
      </div>

      <div id="passkey-modal" class="app-modal-backdrop hidden" aria-hidden="true">
        <div class="app-modal-card" role="dialog" aria-modal="true" aria-labelledby="passkey-modal-title">
          <div class="flex items-center justify-between gap-2">
            <h4 id="passkey-modal-title" class="text-base font-semibold txt-strong">Edit passkey</h4>
            <button id="passkey-modal-delete" type="button" class="btn-destructive h-9 w-9 p-0" aria-label="Delete passkey">
              <img class="h-4 w-4" src="/assets/icons/runtime-ui/icon-trash-destructive-light.svgz" data-svgz-light="/assets/icons/runtime-ui/icon-trash-destructive-light.svgz" data-svgz-dark="/assets/icons/runtime-ui/icon-trash-destructive-dark.svgz" alt="" aria-hidden="true" />
            </button>
          </div>
          <div class="mt-3 space-y-3">
            <div>
              <label class="field-label" for="passkey-modal-label-input">Label</label>
              <input id="passkey-modal-label-input" type="text" maxlength="80" class="field-input h-11 w-full" />
            </div>
            <div>
              <label class="field-label" for="passkey-modal-credential-id">Credential ID</label>
              <input id="passkey-modal-credential-id" type="text" readonly class="field-input h-11 w-full" />
            </div>
            <div class="summary-row">
              <span>Last used</span>
              <span id="passkey-modal-last-used" class="txt-supporting">Never</span>
            </div>
            <div class="summary-row">
              <span>Created</span>
              <span id="passkey-modal-created-at" class="txt-supporting">-</span>
            </div>
            <div class="summary-row">
              <span>AAGUID</span>
              <span id="passkey-modal-aaguid" class="txt-supporting">Unknown</span>
            </div>
            <div class="summary-row">
              <span>Backup eligible (BE)</span>
              <span id="passkey-modal-be" class="txt-supporting">Unknown</span>
            </div>
            <div class="summary-row">
              <span>Backup state (BS)</span>
              <span id="passkey-modal-bs" class="txt-supporting">Unknown</span>
            </div>
          </div>
          <div class="action-pair mt-4" data-action-group="pair">
            <button
              id="passkey-modal-close"
              type="button"
              class="btn-ghost h-11 w-full"
              data-action-role="secondary"
            >
              Close
            </button>
            <button
              id="passkey-modal-save"
              type="button"
              class="btn-primary h-11 w-full"
              data-action-role="primary"
            >
              Save label
            </button>
          </div>
        </div>
      </div>
    </section>
  </div>
</main>
{bottom_nav}
<script>
(() => {{
  const list = document.getElementById('passkeys-list');
  const passkeyStatus = document.getElementById('passkey-status');
  const passkeyCreateButton = document.getElementById('passkey-create');
  const passkeyModal = document.getElementById('passkey-modal');
  const passkeyModalLabelInput = document.getElementById('passkey-modal-label-input');
  const passkeyModalCredentialInput = document.getElementById('passkey-modal-credential-id');
  const passkeyModalLastUsed = document.getElementById('passkey-modal-last-used');
  const passkeyModalCreatedAt = document.getElementById('passkey-modal-created-at');
  const passkeyModalAaguid = document.getElementById('passkey-modal-aaguid');
  const passkeyModalBe = document.getElementById('passkey-modal-be');
  const passkeyModalBs = document.getElementById('passkey-modal-bs');
  const passkeyModalCloseButton = document.getElementById('passkey-modal-close');
  const passkeyModalSaveButton = document.getElementById('passkey-modal-save');
  const passkeyModalDeleteButton = document.getElementById('passkey-modal-delete');
  const passwordForm = document.getElementById('password-change-form');
  const newPasswordInput = document.getElementById('new-password');
  const confirmPasswordInput = document.getElementById('confirm-password');
  const strengthFill = document.getElementById('password-strength-fill');
  const strengthText = document.getElementById('password-strength-text');
  const matchHint = document.getElementById('password-match-hint');
  const passwordStatus = document.getElementById('password-change-status');
  const securityModal = document.getElementById('security-modal');
  const securityModalTitle = document.getElementById('security-modal-title');
  const securityModalMessage = document.getElementById('security-modal-message');
  const securityModalInputWrap = document.getElementById('security-modal-input-wrap');
  const securityModalInput = document.getElementById('security-modal-input');
  const securityModalCancel = document.getElementById('security-modal-cancel');
  const securityModalConfirm = document.getElementById('security-modal-confirm');
  let selectedCredentialId = null;
  let passkeysById = new Map();
  let modalResolver = null;
  const PASSKEY_LABEL_OVERRIDES_KEY = 'bominal.passkey.label_overrides.v1';
  const passkeyLabelOverrides = (() => {{
    try {{
      const raw = window.localStorage.getItem(PASSKEY_LABEL_OVERRIDES_KEY);
      if (!raw) {{
        return new Map();
      }}
      const parsed = JSON.parse(raw);
      if (!parsed || typeof parsed !== 'object') {{
        return new Map();
      }}
      return new Map(Object.entries(parsed).filter(([_, value]) => typeof value === 'string'));
    }} catch (_err) {{
      return new Map();
    }}
  }})();
  const currentThemeMode = () => document.body?.dataset?.themeMode === 'dark' ? 'dark' : 'light';
  const syncThemedSvgzIcons = () => {{
    const mode = currentThemeMode();
    const icons = document.querySelectorAll('img[data-svgz-light][data-svgz-dark]');
    icons.forEach((icon) => {{
      const nextSrc = mode === 'dark' ? icon.dataset.svgzDark : icon.dataset.svgzLight;
      if (!nextSrc || icon.getAttribute('src') === nextSrc) return;
      icon.setAttribute('src', nextSrc);
    }});
  }};

  const requestJson = (url, method, payload) => fetch(url, {{
    method,
    headers: {{ 'Content-Type': 'application/json', 'Accept': 'application/json' }},
    body: payload ? JSON.stringify(payload) : undefined,
  }}).then(async (res) => {{
    const text = await res.text();
    let body = null;
    try {{ body = text ? JSON.parse(text) : null; }} catch (_err) {{}}
    return {{ ok: res.ok, status: res.status, body }};
  }});

  const showStatus = (node, message, kind) => {{
    node.classList.remove('hidden');
    if (kind === 'error') {{
      node.className = 'mt-3 error-card';
    }} else if (kind === 'success') {{
      node.className = 'mt-3 summary-card';
    }} else {{
      node.className = 'mt-3 loading-card';
    }}
    node.textContent = message;
  }};

  const clearStatus = (node) => {{
    node.className = 'mt-3 hidden';
    node.textContent = '';
  }};

  const b64urlToBuffer = (value) => {{
    const padded = (value + '==='.slice((value.length + 3) % 4)).replace(/-/g, '+').replace(/_/g, '/');
    const binary = atob(padded);
    const bytes = new Uint8Array(binary.length);
    for (let i = 0; i < binary.length; i += 1) bytes[i] = binary.charCodeAt(i);
    return bytes.buffer;
  }};

  const bufferToB64url = (buffer) => {{
    const bytes = new Uint8Array(buffer);
    let binary = '';
    for (let i = 0; i < bytes.length; i += 1) binary += String.fromCharCode(bytes[i]);
    return btoa(binary).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
  }};

  const escapeHtml = (value) => String(value || '')
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;');

  const formatTimestamp = (value) => {{
    if (!value) return 'Never';
    const parsed = new Date(value);
    if (Number.isNaN(parsed.getTime())) return String(value);
    return parsed.toLocaleString();
  }};

  const formatBool = (value) => {{
    if (value === true) return 'Yes';
    if (value === false) return 'No';
    return 'Unknown';
  }};

  const detectDeviceLabel = () => {{
    const ua = navigator.userAgent || '';
    const platformRaw = (navigator.userAgentData && navigator.userAgentData.platform) || navigator.platform || '';

    const platform = (() => {{
      const source = `${{platformRaw}} ${{ua}}`;
      if (/iPhone|iPad|iPod/i.test(source)) return 'iOS';
      if (/Android/i.test(source)) return 'Android';
      if (/Mac/i.test(source)) return 'Mac';
      if (/Win/i.test(source)) return 'Windows';
      if (/Linux/i.test(source)) return 'Linux';
      return 'Device';
    }})();

    const browser = (() => {{
      if (/Edg\\//i.test(ua)) return 'Edge';
      if (/Firefox\\//i.test(ua)) return 'Firefox';
      if (/Chrome\\//i.test(ua) && !/Edg\\//i.test(ua)) return 'Chrome';
      if (/Safari\\//i.test(ua) && !/Chrome\\//i.test(ua)) return 'Safari';
      return 'Browser';
    }})();

    return `${{platform}} ${{browser}}`;
  }};

  const safeDetectDeviceLabel = () => {{
    try {{
      return detectDeviceLabel();
    }} catch (_err) {{
      return 'This device';
    }}
  }};

  const persistPasskeyLabelOverrides = () => {{
    try {{
      const serializable = Object.fromEntries(passkeyLabelOverrides.entries());
      window.localStorage.setItem(PASSKEY_LABEL_OVERRIDES_KEY, JSON.stringify(serializable));
    }} catch (_err) {{}}
  }};

  const effectivePasskeyLabel = (passkey) => {{
    if (!passkey || !passkey.credential_id) {{
      return 'Unnamed passkey';
    }}
    const override = passkeyLabelOverrides.get(passkey.credential_id);
    if (override && override.trim()) {{
      return override.trim();
    }}
    if (passkey.friendly_name && String(passkey.friendly_name).trim()) {{
      return String(passkey.friendly_name).trim();
    }}
    return 'Unnamed passkey';
  }};

  const serializeRegisterCredential = (credential) => {{
    const response = credential.response;
    const transports = typeof response.getTransports === 'function' ? response.getTransports() : null;
    const serializedResponse = {{
      attestationObject: bufferToB64url(response.attestationObject),
      clientDataJSON: bufferToB64url(response.clientDataJSON),
    }};
    if (Array.isArray(transports) && transports.length) {{
      serializedResponse.transports = transports;
    }}
    return {{
      id: credential.id,
      rawId: bufferToB64url(credential.rawId),
      type: credential.type,
      response: serializedResponse,
      clientExtensionResults: credential.getClientExtensionResults ? credential.getClientExtensionResults() : {{}},
    }};
  }};

  const scorePassword = (value) => {{
    let score = 0;
    if (value.length >= 10) score += 1;
    if (value.length >= 14) score += 1;
    if (/[a-z]/.test(value)) score += 1;
    if (/[A-Z]/.test(value)) score += 1;
    if (/[0-9]/.test(value)) score += 1;
    if (/[^A-Za-z0-9]/.test(value)) score += 1;
    return score;
  }};

  const renderStrength = () => {{
    const score = scorePassword(newPasswordInput.value);
    const percent = Math.round((score / 6) * 100);
    strengthFill.style.width = `${{percent}}%`;
    strengthFill.classList.remove('bg-rose-400', 'bg-amber-400', 'bg-emerald-500');
    if (score <= 2) {{
      strengthFill.classList.add('bg-rose-400');
      strengthText.textContent = 'Strength: weak';
    }} else if (score <= 4) {{
      strengthFill.classList.add('bg-amber-400');
      strengthText.textContent = 'Strength: medium';
    }} else {{
      strengthFill.classList.add('bg-emerald-500');
      strengthText.textContent = 'Strength: strong';
    }}
    return score;
  }};

  const renderMatch = () => {{
    const confirmValue = confirmPasswordInput.value;
    const matches = newPasswordInput.value === confirmValue && confirmValue.length > 0;
    matchHint.classList.remove('txt-critical', 'txt-positive');
    if (!confirmValue) {{
      matchHint.textContent = 'Passwords must match.';
      return false;
    }}
    if (matches) {{
      matchHint.textContent = 'Passwords match.';
      matchHint.classList.add('txt-positive');
      return true;
    }}
    matchHint.textContent = 'Passwords do not match.';
    matchHint.classList.add('txt-critical');
    return false;
  }};

  const MODAL_LAYER_BASE = 70;
  let modalLayerCounter = 0;

  const bringModalToBody = (modalNode) => {{
    if (!modalNode || !document.body) {{
      return;
    }}
    if (modalNode.parentElement !== document.body) {{
      document.body.appendChild(modalNode);
    }}
  }};

  const openModalLayer = (modalNode) => {{
    if (!modalNode) {{
      return;
    }}
    bringModalToBody(modalNode);
    modalLayerCounter += 1;
    modalNode.style.zIndex = String(MODAL_LAYER_BASE + modalLayerCounter);
    modalNode.classList.remove('hidden');
    modalNode.setAttribute('aria-hidden', 'false');
  }};

  const closeModalLayer = (modalNode) => {{
    if (!modalNode) {{
      return;
    }}
    modalNode.classList.add('hidden');
    modalNode.setAttribute('aria-hidden', 'true');
    modalNode.style.removeProperty('z-index');
    if (!document.querySelector('.app-modal-backdrop:not(.hidden)')) {{
      modalLayerCounter = 0;
    }}
  }};

  const topVisibleModal = () => {{
    const visible = Array.from(document.querySelectorAll('.app-modal-backdrop:not(.hidden)'));
    if (!visible.length) {{
      return null;
    }}
    return visible.reduce((currentTop, candidate) => {{
      const currentZ = Number(currentTop.style.zIndex || MODAL_LAYER_BASE);
      const candidateZ = Number(candidate.style.zIndex || MODAL_LAYER_BASE);
      return candidateZ >= currentZ ? candidate : currentTop;
    }});
  }};

  const closeSecurityModal = (result) => {{
    if (!modalResolver || !securityModal) {{
      return;
    }}
    const resolve = modalResolver;
    modalResolver = null;
    closeModalLayer(securityModal);
    if (securityModalInput) {{
      securityModalInput.value = '';
    }}
    resolve(result);
  }};

  const openSecurityModal = (options) => new Promise((resolve) => {{
    if (!securityModal || !securityModalTitle || !securityModalMessage || !securityModalInputWrap || !securityModalInput || !securityModalCancel || !securityModalConfirm) {{
      resolve({{ confirmed: false, value: '' }});
      return;
    }}

    securityModalTitle.textContent = options.title || 'Confirm action';
    securityModalMessage.textContent = options.message || '';
    securityModalConfirm.textContent = options.confirmText || 'Confirm';
    securityModalInputWrap.classList.toggle('hidden', !options.withPassword);
    securityModalInput.value = '';
    openModalLayer(securityModal);
    modalResolver = resolve;

    requestAnimationFrame(() => {{
      if (options.withPassword) {{
        securityModalInput.focus();
      }} else {{
        securityModalConfirm.focus();
      }}
    }});
  }});

  const closePasskeyModal = () => {{
    if (!passkeyModal) {{
      return;
    }}
    closeModalLayer(passkeyModal);
  }};

  const openPasskeyModal = () => {{
    const selected = selectedCredentialId ? passkeysById.get(selectedCredentialId) : null;
    if (!selected) {{
      showStatus(passkeyStatus, 'Select a passkey to edit.', 'error');
      return;
    }}
    if (!passkeyModal || !passkeyModalLabelInput || !passkeyModalCredentialInput || !passkeyModalLastUsed || !passkeyModalCreatedAt || !passkeyModalAaguid || !passkeyModalBe || !passkeyModalBs) {{
      showStatus(passkeyStatus, 'Passkey editor is unavailable.', 'error');
      return;
    }}

    passkeyModalLabelInput.value = effectivePasskeyLabel(selected);
    passkeyModalCredentialInput.value = selected.credential_id || '';
    passkeyModalLastUsed.textContent = selected.last_used_at ? formatTimestamp(selected.last_used_at) : 'Never';
    passkeyModalCreatedAt.textContent = formatTimestamp(selected.created_at);
    passkeyModalAaguid.textContent = selected.aaguid || 'Unknown';
    passkeyModalBe.textContent = formatBool(selected.backup_eligible);
    passkeyModalBs.textContent = formatBool(selected.backup_state);
    openModalLayer(passkeyModal);
    requestAnimationFrame(() => {{
      passkeyModalLabelInput.focus();
      passkeyModalLabelInput.select();
    }});
  }};

  const syncPasskeySelection = () => {{
    let selectedExists = false;
    list.querySelectorAll('[data-credential-id]').forEach((card) => {{
      const selected = card.dataset.credentialId === selectedCredentialId;
      card.classList.toggle('passkey-card-selected', selected);
      card.setAttribute('aria-pressed', selected ? 'true' : 'false');
      if (selected) {{
        selectedExists = true;
      }}
    }});
    if (!selectedExists) {{
      selectedCredentialId = null;
    }}
  }};

  const load = () => fetch('/api/auth/passkeys', {{ headers: {{ Accept: 'application/json' }} }})
    .then((res) => res.json().then((json) => [res.ok, json]))
    .then(([ok, data]) => {{
      if (!ok) {{
        passkeysById = new Map();
        selectedCredentialId = null;
        syncPasskeySelection();
        list.innerHTML = `<div class="error-card">Failed to load passkeys. request_id: ${{data.request_id || 'n/a'}}</div>`;
        return;
      }}
      const passkeys = data.passkeys || [];
      passkeysById = new Map(passkeys.map((item) => [item.credential_id, item]));
      if (!passkeys.length) {{
        selectedCredentialId = null;
        syncPasskeySelection();
        list.innerHTML = '<div class="empty-card">No passkeys registered.</div>';
        return;
      }}
      if (selectedCredentialId && !passkeys.some((item) => item.credential_id === selectedCredentialId)) {{
        selectedCredentialId = null;
      }}
      list.innerHTML = passkeys.map((item) => `
        <button type="button" class="summary-card passkey-card w-full text-left" data-credential-id="${{item.credential_id}}" aria-pressed="false">
          <div class="flex items-center justify-between gap-2 text-sm">
            <span>${{escapeHtml(effectivePasskeyLabel(item))}}</span>
            <span class="txt-supporting">${{item.last_used_at ? `Used ${{formatTimestamp(item.last_used_at)}}` : 'Never used'}}</span>
          </div>
        </button>
      `).join('');
      list.querySelectorAll('[data-credential-id]').forEach((card) => {{
        card.addEventListener('click', () => {{
          selectedCredentialId = card.dataset.credentialId || null;
          syncPasskeySelection();
          openPasskeyModal();
        }});
      }});
      syncPasskeySelection();
    }});

  const deleteSelectedPasskey = async () => {{
    if (!selectedCredentialId) {{
      return;
    }}
    const modalResult = await openSecurityModal({{
      title: 'Delete passkey',
      message: `Delete selected passkey (${{selectedCredentialId}})? This action cannot be undone.`,
      confirmText: 'Delete',
      withPassword: false,
    }});
    if (!modalResult.confirmed) {{
      return;
    }}
    clearStatus(passkeyStatus);
    if (passkeyModalDeleteButton) {{
      passkeyModalDeleteButton.disabled = true;
    }}
    const result = await requestJson(`/api/auth/passkeys/${{selectedCredentialId}}`, 'DELETE');
    if (!result.ok) {{
      const requestId = result.body && result.body.request_id ? ` (request_id: ${{result.body.request_id}})` : '';
      showStatus(passkeyStatus, `Passkey deletion failed${{requestId}}`, 'error');
      syncPasskeySelection();
      if (passkeyModalDeleteButton) {{
        passkeyModalDeleteButton.disabled = false;
      }}
      return;
    }}
    passkeyLabelOverrides.delete(selectedCredentialId);
    persistPasskeyLabelOverrides();
    selectedCredentialId = null;
    closePasskeyModal();
    showStatus(passkeyStatus, 'Passkey deleted successfully.', 'success');
    await load();
    if (passkeyModalDeleteButton) {{
      passkeyModalDeleteButton.disabled = false;
    }}
  }};

  const saveSelectedPasskeyLabel = async () => {{
    if (!selectedCredentialId || !passkeyModalLabelInput) {{
      return;
    }}
    const friendlyName = passkeyModalLabelInput.value.trim();
    if (!friendlyName) {{
      showStatus(passkeyStatus, 'Passkey label is required.', 'error');
      passkeyModalLabelInput.focus();
      return;
    }}
    clearStatus(passkeyStatus);
    if (passkeyModalSaveButton) {{
      passkeyModalSaveButton.disabled = true;
    }}
    try {{
      const result = await requestJson(`/api/auth/passkeys/${{selectedCredentialId}}`, 'PATCH', {{
        friendly_name: friendlyName,
      }});
      if (!result.ok) {{
        const requestId = result.body && result.body.request_id ? ` (request_id: ${{result.body.request_id}})` : '';
        showStatus(passkeyStatus, `Passkey label update failed${{requestId}}`, 'error');
        return;
      }}
      passkeyLabelOverrides.delete(selectedCredentialId);
      persistPasskeyLabelOverrides();
      showStatus(passkeyStatus, 'Passkey label updated successfully.', 'success');
      syncPasskeySelection();
      await load();
    }} catch (_err) {{
      showStatus(passkeyStatus, 'Passkey label update failed.', 'error');
    }} finally {{
      if (passkeyModalSaveButton) {{
        passkeyModalSaveButton.disabled = false;
      }}
    }}
  }};

  const createPasskey = async () => {{
    clearStatus(passkeyStatus);
    if (!window.PublicKeyCredential || !navigator.credentials) {{
      showStatus(passkeyStatus, 'Passkeys are unavailable in this browser.', 'error');
      return;
    }}

    if (passkeyCreateButton) {{
      passkeyCreateButton.disabled = true;
    }}
    try {{
      const friendlyName = safeDetectDeviceLabel();
      const start = await requestJson('/api/auth/passkeys/register/start', 'POST', {{
        friendly_name: friendlyName,
      }});
      if (!start.ok || !start.body || !start.body.options || !start.body.flow_id) {{
        const requestId = start.body && start.body.request_id ? ` (request_id: ${{start.body.request_id}})` : '';
        showStatus(passkeyStatus, `Passkey setup failed${{requestId}}`, 'error');
        return;
      }}

      const options = typeof structuredClone === 'function'
        ? structuredClone(start.body.options)
        : JSON.parse(JSON.stringify(start.body.options));
      options.publicKey.challenge = b64urlToBuffer(options.publicKey.challenge);
      if (options.publicKey.user && options.publicKey.user.id) {{
        options.publicKey.user.id = b64urlToBuffer(options.publicKey.user.id);
      }}
      if (Array.isArray(options.publicKey.excludeCredentials)) {{
        options.publicKey.excludeCredentials = options.publicKey.excludeCredentials.map((item) => ({{ ...item, id: b64urlToBuffer(item.id) }}));
      }}

      const credential = await navigator.credentials.create(options);
      if (!credential) {{
        showStatus(passkeyStatus, 'Passkey creation was cancelled.', 'error');
        return;
      }}

      const finish = await requestJson('/api/auth/passkeys/register/finish', 'POST', {{
        flow_id: start.body.flow_id,
        credential: serializeRegisterCredential(credential),
      }});
      if (!finish.ok) {{
        const requestId = finish.body && finish.body.request_id ? ` (request_id: ${{finish.body.request_id}})` : '';
        showStatus(passkeyStatus, `Passkey setup failed${{requestId}}`, 'error');
        return;
      }}

      showStatus(passkeyStatus, 'Passkey created successfully.', 'success');
      load();
    }} catch (err) {{
      const errName = err && typeof err === 'object' && err.name ? String(err.name) : '';
      if (err && typeof err === 'object' && err.name === 'SecurityError') {{
        showStatus(passkeyStatus, 'Passkeys are unavailable on this host. Use localhost for WebAuthn.', 'error');
      }} else if (err && typeof err === 'object' && err.name === 'NotAllowedError') {{
        showStatus(passkeyStatus, 'Passkey creation was cancelled or blocked by the browser.', 'error');
      }} else if (err && typeof err === 'object' && err.name === 'InvalidStateError') {{
        showStatus(passkeyStatus, 'This authenticator is already registered for your account.', 'error');
      }} else {{
        const suffix = errName ? ` (${{errName}})` : '';
        showStatus(passkeyStatus, `Passkey creation failed${{suffix}}.`, 'error');
      }}
    }} finally {{
      if (passkeyCreateButton) {{
        passkeyCreateButton.disabled = false;
      }}
    }}
  }};

  if (passkeyCreateButton) {{
    passkeyCreateButton.addEventListener('click', async (event) => {{
      event.preventDefault();
      const modalResult = await openSecurityModal({{
        title: 'Create passkey',
        message: 'Passkeys are faster to use and resistant to phishing because they do not require typing your password. Create a passkey on this device now?',
        confirmText: 'Continue',
        withPassword: false,
      }});
      if (!modalResult.confirmed) {{
        return;
      }}
      createPasskey();
    }});
  }}
  if (passkeyModalCloseButton) {{
    passkeyModalCloseButton.addEventListener('click', closePasskeyModal);
  }}
  if (passkeyModalSaveButton) {{
    passkeyModalSaveButton.addEventListener('click', saveSelectedPasskeyLabel);
  }}
  if (passkeyModalDeleteButton) {{
    passkeyModalDeleteButton.addEventListener('click', deleteSelectedPasskey);
  }}
  if (passkeyModal) {{
    passkeyModal.addEventListener('click', (event) => {{
      if (event.target === passkeyModal) {{
        closePasskeyModal();
      }}
    }});
  }}

  if (securityModalCancel) {{
    securityModalCancel.addEventListener('click', () => {{
      closeSecurityModal({{ confirmed: false, value: '' }});
    }});
  }}

  if (securityModalConfirm) {{
    securityModalConfirm.addEventListener('click', () => {{
      const requiresPassword = securityModalInputWrap && !securityModalInputWrap.classList.contains('hidden');
      if (requiresPassword) {{
        const value = securityModalInput.value;
        if (!value) {{
          securityModalInput.focus();
          return;
        }}
        closeSecurityModal({{ confirmed: true, value }});
        return;
      }}
      closeSecurityModal({{ confirmed: true, value: '' }});
    }});
  }}

  if (securityModal) {{
    securityModal.addEventListener('click', (event) => {{
      if (event.target === securityModal) {{
        closeSecurityModal({{ confirmed: false, value: '' }});
      }}
    }});
  }}

  document.addEventListener('keydown', (event) => {{
    if (event.key !== 'Escape') {{
      return;
    }}
    const modal = topVisibleModal();
    if (!modal) {{
      return;
    }}
    event.preventDefault();
    if (modal === securityModal) {{
      closeSecurityModal({{ confirmed: false, value: '' }});
      return;
    }}
    if (modal === passkeyModal) {{
      closePasskeyModal();
    }}
  }});

  newPasswordInput.addEventListener('input', () => {{
    renderStrength();
    renderMatch();
  }});
  confirmPasswordInput.addEventListener('input', renderMatch);

  passwordForm.addEventListener('submit', async (event) => {{
    event.preventDefault();
    clearStatus(passwordStatus);

    const score = renderStrength();
    const matches = renderMatch();
    if (!matches) {{
      showStatus(passwordStatus, 'New password and confirmation must match.', 'error');
      return;
    }}
    if (score < 4) {{
      showStatus(passwordStatus, 'Password strength is too weak. Use a longer mixed password.', 'error');
      return;
    }}

    const modalResult = await openSecurityModal({{
      title: 'Confirm password update',
      message: 'Enter your current password to apply this change.',
      confirmText: 'Update',
      withPassword: true,
    }});
    if (!modalResult.confirmed) {{
      return;
    }}
    const currentPassword = modalResult.value;
    if (!currentPassword) {{
      showStatus(passwordStatus, 'Current password is required to update password.', 'error');
      return;
    }}

    const result = await requestJson('/api/auth/password/change', 'POST', {{
      current_password: currentPassword,
      new_password: newPasswordInput.value,
    }});
    if (!result.ok) {{
      const requestId = result.body && result.body.request_id ? ` (request_id: ${{result.body.request_id}})` : '';
      showStatus(passwordStatus, `Password update failed${{requestId}}`, 'error');
      return;
    }}

    showStatus(passwordStatus, 'Password updated successfully.', 'success');
    newPasswordInput.value = '';
    confirmPasswordInput.value = '';
    renderStrength();
    renderMatch();
  }});

  renderStrength();
  renderMatch();
  syncThemedSvgzIcons();
  if (document.body && typeof MutationObserver === 'function') {{
    const themeObserver = new MutationObserver(() => syncThemedSvgzIcons());
    themeObserver.observe(document.body, {{ attributes: true, attributeFilter: ['data-theme-mode'] }});
  }}
  load();
}})();
</script>"#
    )
}
