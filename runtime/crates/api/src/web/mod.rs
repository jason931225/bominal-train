pub mod auth;
pub mod dashboard;
pub mod home;
pub mod layout;

pub use auth::*;
pub use dashboard::*;
pub use home::*;
pub use layout::*;

use bominal_ui::{
    AdminSection, DashboardSection, render_admin_bottom_nav, render_admin_sidebar,
    render_dashboard_bottom_nav,
};

pub fn render_dashboard_settings_providers(email: &str) -> String {
    let topbar = app_shell_topbar("Settings", &format!("Signed in as {}", html_escape(email)));
    let sidebar = dashboard_desktop_sidebar("security");
    let bottom_nav = render_dashboard_bottom_nav(DashboardSection::Settings);
    let settings_tabs = dashboard_settings_tabs("provider");
    let mut html = String::new();
    html.push_str(&topbar);
    html.push_str(
        r#"<main class="mx-auto w-full max-w-[480px] px-4 pb-24 pt-4 md:max-w-7xl md:px-6 md:pt-8">
  <div class="md:grid md:grid-cols-[220px_minmax(0,1fr)] md:items-start md:gap-4">"#,
    );
    html.push_str(&sidebar);
    html.push_str(r#"<section class="space-y-4">"#);
    html.push_str(&settings_tabs);
    html.push_str(
        r#"
      <section class="glass-card rounded-[22px] p-5">
        <h2 class="text-lg font-semibold txt-strong">Provider Authentication</h2>
        <p class="mt-1 text-sm txt-supporting">Select a provider card to open its authentication form.</p>
        <div id="provider-security-preflight" class="mt-3 grid grid-cols-2 gap-2"><div class="loading-card col-span-2">Loading status...</div></div>
      </section>

      <div id="provider-security-status" class="hidden"></div>
    </section>
  </div>
</main>
<div id="provider-auth-modal" class="app-modal-backdrop hidden" aria-hidden="true">
  <div class="app-modal-card" role="dialog" aria-modal="true" aria-labelledby="provider-auth-modal-title">
    <div class="flex items-center justify-between gap-2">
      <h3 id="provider-auth-modal-title" class="text-base font-semibold txt-strong">Provider Authentication</h3>
      <button id="provider-auth-modal-delete" type="button" class="btn-destructive h-9 w-9 p-0" aria-label="Delete credentials">
        <img class="h-4 w-4" src="/assets/icons/runtime-ui/icon-trash-destructive-light.svgz" data-svgz-light="/assets/icons/runtime-ui/icon-trash-destructive-light.svgz" data-svgz-dark="/assets/icons/runtime-ui/icon-trash-destructive-dark.svgz" alt="" aria-hidden="true" />
      </button>
    </div>
    <p class="mt-1 text-xs txt-supporting"><span id="provider-auth-modal-provider">Provider</span> credentials are encrypted server-side.</p>
    <form id="provider-auth-form" class="mt-3 space-y-3">
      <label class="field-label" for="provider-auth-account">Account identifier</label>
      <input id="provider-auth-account" type="text" autocomplete="username" class="field-input h-11 w-full" />
      <label class="field-label" for="provider-auth-password">Password</label>
      <input id="provider-auth-password" type="password" autocomplete="current-password" class="field-input h-11 w-full" />
      <div class="grid grid-cols-2 gap-2" data-action-group="pair">
        <button id="provider-auth-modal-cancel" type="button" class="btn-ghost h-11 w-full">Cancel</button>
        <button id="provider-auth-modal-save" type="submit" class="btn-primary h-11 w-full">Save</button>
      </div>
    </form>
  </div>
</div>
<script>
(() => {
  const preflightNode = document.getElementById('provider-security-preflight');
  const statusNode = document.getElementById('provider-security-status');
  const authModal = document.getElementById('provider-auth-modal');
  const authModalTitle = document.getElementById('provider-auth-modal-title');
  const authModalProvider = document.getElementById('provider-auth-modal-provider');
  const authModalDelete = document.getElementById('provider-auth-modal-delete');
  const authModalCancel = document.getElementById('provider-auth-modal-cancel');
  const authForm = document.getElementById('provider-auth-form');
  const authAccountInput = document.getElementById('provider-auth-account');
  const authPasswordInput = document.getElementById('provider-auth-password');

  const requestJson = async (url, method, payload) => {
    const response = await fetch(url, {
      method: method || 'GET',
      headers: { 'Content-Type': 'application/json', 'Accept': 'application/json' },
      body: payload ? JSON.stringify(payload) : undefined,
    });
    const text = await response.text();
    let body = null;
    try { body = text ? JSON.parse(text) : null; } catch (_err) {}
    return { ok: response.ok, status: response.status, body };
  };

  const escapeHtml = (value) => String(value || '')
    .replaceAll('&', '&amp;')
    .replaceAll('<', '&lt;')
    .replaceAll('>', '&gt;')
    .replaceAll('"', '&quot;')
    .replaceAll("'", '&#39;');

  const apiErrorMessage = (response, fallback) => {
    const body = response && response.body && typeof response.body === 'object' ? response.body : {};
    const message = typeof body.message === 'string' && body.message.trim()
      ? body.message.trim()
      : fallback;
    const requestId = typeof body.request_id === 'string' && body.request_id.trim()
      ? ` (request_id: ${body.request_id.trim()})`
      : '';
    return `${message}${requestId}`;
  };

  const providerOrder = ['srt', 'ktx'];
  const MODAL_LAYER_BASE = 70;
  let modalLayerCounter = 0;
  let activeProvider = null;
  let latestProvidersByName = new Map();
  const currentThemeMode = () => document.body?.dataset?.themeMode === 'dark' ? 'dark' : 'light';
  const syncThemedSvgzIcons = (rootNode) => {
    const mode = currentThemeMode();
    const root = rootNode && typeof rootNode.querySelectorAll === 'function' ? rootNode : document;
    const icons = root.querySelectorAll('img[data-svgz-light][data-svgz-dark]');
    icons.forEach((icon) => {
      const nextSrc = mode === 'dark' ? icon.dataset.svgzDark : icon.dataset.svgzLight;
      if (!nextSrc || icon.getAttribute('src') === nextSrc) return;
      icon.setAttribute('src', nextSrc);
    });
  };

  const showStatus = (kind, message) => {
    statusNode.classList.remove('hidden');
    statusNode.className = kind === 'error' ? 'error-card' : 'summary-card';
    statusNode.textContent = message;
  };

  const bringModalToBody = (modalNode) => {
    if (!modalNode || !document.body) return;
    if (modalNode.parentElement !== document.body) {
      document.body.appendChild(modalNode);
    }
  };

  const openModalLayer = (modalNode) => {
    if (!modalNode) return;
    bringModalToBody(modalNode);
    modalLayerCounter += 1;
    modalNode.style.zIndex = String(MODAL_LAYER_BASE + modalLayerCounter);
    modalNode.classList.remove('hidden');
    modalNode.setAttribute('aria-hidden', 'false');
  };

  const closeModalLayer = (modalNode) => {
    if (!modalNode) return;
    modalNode.classList.add('hidden');
    modalNode.setAttribute('aria-hidden', 'true');
    modalNode.style.removeProperty('z-index');
    if (!document.querySelector('.app-modal-backdrop:not(.hidden)')) {
      modalLayerCounter = 0;
    }
  };

  const normalizeProvider = (value) => {
    const normalized = String(value || '').trim().toLowerCase();
    return providerOrder.includes(normalized) ? normalized : null;
  };

  const activeProviderRecord = () => {
    if (!activeProvider) return null;
    return latestProvidersByName.get(activeProvider) || null;
  };

  const providerAuthProbeStatus = (provider) => {
    if (!provider || typeof provider !== 'object') return '';
    const value = typeof provider.auth_probe_status === 'string'
      ? provider.auth_probe_status.trim().toLowerCase()
      : '';
    return value === 'error' || value === 'success' || value === 'skipped' ? value : '';
  };

  const providerStatusMessage = (provider) => {
    if (!provider || typeof provider !== 'object') return '';
    if (typeof provider.auth_probe_message === 'string' && provider.auth_probe_message.trim()) {
      return provider.auth_probe_message.trim();
    }
    if (typeof provider.error === 'string' && provider.error.trim()) return provider.error.trim();
    if (typeof provider.debug === 'string' && provider.debug.trim()) return provider.debug.trim();
    if (typeof provider.message === 'string' && provider.message.trim()) return provider.message.trim();
    return '';
  };

  const statusIcon = (kind, ready, hasError) => {
    const title = kind === 'credentials' ? 'Credentials' : 'Payment';
    const state = hasError ? 'error' : (ready ? 'ready' : 'missing');
    const iconPrefix = kind === 'credentials' ? 'provider-status-credentials' : 'provider-status-payment';
    const lightVariant = hasError
      ? `${iconPrefix}-red.svgz`
      : ready
        ? `${iconPrefix}-green.svgz`
        : `${iconPrefix}-gray-light.svgz`;
    const darkVariant = hasError
      ? `${iconPrefix}-red.svgz`
      : ready
        ? `${iconPrefix}-green.svgz`
        : `${iconPrefix}-gray-dark.svgz`;
    const src = currentThemeMode() === 'dark' ? darkVariant : lightVariant;
    return `
      <span class="provider-status-chip" title="${title}: ${state}" aria-label="${title}: ${state}">
        <img class="status-icon" src="/assets/icons/runtime-ui/${src}" data-svgz-light="/assets/icons/runtime-ui/${lightVariant}" data-svgz-dark="/assets/icons/runtime-ui/${darkVariant}" alt="" aria-hidden="true" />
      </span>
    `;
  };

  const closeAuthModal = () => {
    closeModalLayer(authModal);
    activeProvider = null;
    if (authForm) authForm.reset();
  };

  const openAuthModal = (provider) => {
    const normalizedProvider = normalizeProvider(provider);
    if (!normalizedProvider) return;
    activeProvider = normalizedProvider;
    if (authModalTitle) authModalTitle.textContent = `${normalizedProvider.toUpperCase()} Authentication`;
    if (authModalProvider) authModalProvider.textContent = normalizedProvider.toUpperCase();
    if (authForm) authForm.reset();
    openModalLayer(authModal);
    requestAnimationFrame(() => {
      if (authAccountInput) authAccountInput.focus();
    });
  };

  const deleteProviderCredentials = async () => {
    const selected = activeProviderRecord();
    if (!selected || !activeProvider) return;
    const deletedProvider = activeProvider;
    const prompt = `Delete saved ${deletedProvider.toUpperCase()} credentials?`;
    if (!window.confirm(prompt)) return;
    const response = await requestJson(`/api/train/providers/${deletedProvider}/credentials`, 'DELETE');
    if (!response.ok) {
      showStatus(
        'error',
        apiErrorMessage(response, `Could not delete ${deletedProvider.toUpperCase()} credentials.`),
      );
      return;
    }
    closeAuthModal();
    showStatus('success', `${deletedProvider.toUpperCase()}: Credentials removed.`);
    await loadPreflight();
  };

  const loadPreflight = async () => {
    const response = await requestJson('/api/train/preflight');
    if (!response.ok) {
      preflightNode.innerHTML = `<div class="error-card col-span-2">${escapeHtml(apiErrorMessage(response, 'Could not load readiness.'))}</div>`;
      return;
    }
    const providers = response.body && Array.isArray(response.body.providers) ? response.body.providers : [];
    latestProvidersByName = new Map(
      providers
        .map((provider) => {
          const key = normalizeProvider(provider.provider);
          return key ? [key, provider] : null;
        })
        .filter((entry) => !!entry),
    );

    preflightNode.innerHTML = providers.map((provider) => {
      const providerKey = normalizeProvider(provider.provider);
      const statusMessage = providerStatusMessage(provider);
      const probeStatus = providerAuthProbeStatus(provider);
      const hasError = probeStatus === 'error';
      return `
        <button type="button" class="summary-card provider-select-card p-3 w-full text-left" data-provider-open="${providerKey || ''}">
          <span class="flex h-8 items-center justify-between gap-2">
            <span class="txt-strong">${provider.provider.toUpperCase()}</span>
            <span class="provider-status-group">
              ${statusIcon('credentials', provider.credentials_ready, hasError)}
            </span>
          </span>
          ${statusMessage ? `<span class="support-row mt-2 block">${escapeHtml(statusMessage)}</span>` : ''}
        </button>
      `;
    }).join('') || '<div class="empty-card col-span-2">No provider data.</div>';

    Array.from(preflightNode.querySelectorAll('[data-provider-open]')).forEach((button) => {
      button.addEventListener('click', () => {
        const provider = normalizeProvider(button.getAttribute('data-provider-open'));
        if (!provider) return;
        openAuthModal(provider);
      });
    });
    syncThemedSvgzIcons(preflightNode);
  };

  if (authModalCancel) {
    authModalCancel.addEventListener('click', () => {
      closeAuthModal();
    });
  }

  if (authModalDelete) {
    authModalDelete.addEventListener('click', async () => {
      await deleteProviderCredentials();
    });
  }

  if (authForm) {
    authForm.addEventListener('submit', async (event) => {
      event.preventDefault();
      if (!activeProvider) return;
      const submittedProvider = activeProvider;
      const account = String(authAccountInput && authAccountInput.value ? authAccountInput.value : '').trim();
      const password = String(authPasswordInput && authPasswordInput.value ? authPasswordInput.value : '');
      const response = await requestJson(`/api/train/providers/${submittedProvider}/credentials`, 'PUT', {
        account_identifier: account,
        password,
      });
      if (!response.ok) {
        showStatus('error', apiErrorMessage(response, `Could not save ${submittedProvider.toUpperCase()} credentials.`));
        return;
      }
      const result = response.body && typeof response.body === 'object' ? response.body : {};
      const probeStatus = typeof result.auth_probe_status === 'string'
        ? result.auth_probe_status.trim().toLowerCase()
        : '';
      const probeMessage = typeof result.auth_probe_message === 'string' && result.auth_probe_message.trim()
        ? result.auth_probe_message.trim()
        : '';
      closeAuthModal();
      await loadPreflight();
      if (probeStatus === 'error') {
        showStatus('error', probeMessage || `${submittedProvider.toUpperCase()}: Authentication failed.`);
        return;
      }
      showStatus('success', probeMessage || `${submittedProvider.toUpperCase()}: Successfully authenticated.`);
    });
  }

  if (authModal) {
    authModal.addEventListener('click', (event) => {
      if (event.target === authModal) {
        closeAuthModal();
      }
    });
  }

  document.addEventListener('keydown', (event) => {
    if (event.key !== 'Escape') return;
    if (!authModal || authModal.classList.contains('hidden')) return;
    closeAuthModal();
  });

  syncThemedSvgzIcons();
  if (document.body && typeof MutationObserver === 'function') {
    const themeObserver = new MutationObserver(() => syncThemedSvgzIcons());
    themeObserver.observe(document.body, { attributes: true, attributeFilter: ['data-theme-mode'] });
  }
  loadPreflight();
})();
</script>"#,
    );
    html.push_str(&bottom_nav);
    html
}

pub fn render_dashboard_payment(email: &str) -> String {
    let topbar = app_shell_topbar("Settings", &format!("Signed in as {}", html_escape(email)));
    let sidebar = dashboard_desktop_sidebar("security");
    let bottom_nav = render_dashboard_bottom_nav(DashboardSection::Settings);
    let settings_tabs = dashboard_settings_tabs("payment");
    let mut html = String::new();
    html.push_str(&topbar);
    html.push_str(
        r#"<main class="mx-auto w-full max-w-[480px] px-4 pb-24 pt-4 md:max-w-7xl md:px-6 md:pt-8">
  <div class="md:grid md:grid-cols-[220px_minmax(0,1fr)] md:items-start md:gap-4">"#,
    );
    html.push_str(&sidebar);
    html.push_str(r#"<section class="space-y-4">"#);
    html.push_str(&settings_tabs);
    html.push_str(
        r#"
      <section class="glass-card rounded-[22px] p-5">
        <div class="flex items-center justify-between gap-2">
          <h3 class="text-base font-semibold txt-strong">Saved cards</h3>
          <button
            id="payment-modal-open"
            type="button"
            class="btn-chip inline-flex h-9 w-9 items-center justify-center rounded-xl p-0"
            aria-label="Add payment card"
            title="Add payment card"
          >
            <img class="h-4 w-4 icon-muted" src="/assets/icons/runtime-ui/icon-plus-btn-chip-light.svgz" data-svgz-light="/assets/icons/runtime-ui/icon-plus-btn-chip-light.svgz" data-svgz-dark="/assets/icons/runtime-ui/icon-plus-btn-chip-dark.svgz" alt="" aria-hidden="true" />
          </button>
        </div>
        <div id="payment-card-list" class="mt-3 space-y-2"><div class="loading-card">Loading saved cards...</div></div>
      </section>

      <div id="payment-status" class="hidden"></div>
    </section>
  </div>
</main>
<div id="payment-modal" class="app-modal-backdrop hidden" aria-hidden="true">
  <div class="app-modal-card" role="dialog" aria-modal="true" aria-labelledby="payment-modal-title">
    <div class="flex items-center justify-between gap-2">
      <h4 id="payment-modal-title" class="text-base font-semibold txt-strong">Payment information</h4>
      <button id="payment-modal-delete" type="button" class="btn-destructive h-9 w-9 p-0" aria-label="Delete card">
        <img class="h-4 w-4" src="/assets/icons/runtime-ui/icon-trash-destructive-light.svgz" data-svgz-light="/assets/icons/runtime-ui/icon-trash-destructive-light.svgz" data-svgz-dark="/assets/icons/runtime-ui/icon-trash-destructive-dark.svgz" alt="" aria-hidden="true" />
      </button>
    </div>
    <p id="payment-modal-selected-card" class="mt-1 text-xs txt-supporting">Adding a new card</p>
    <form id="payment-form-universal" class="mt-3 space-y-3">
      <label class="field-label" for="payment-pan">pan_ev</label>
      <input id="payment-pan" type="text" class="field-input h-11 w-full" autocomplete="off" />

      <label class="field-label" for="payment-expiry-month">expiry_month_ev</label>
      <input id="payment-expiry-month" type="text" class="field-input h-11 w-full" autocomplete="off" />

      <label class="field-label" for="payment-expiry-year">expiry_year_ev</label>
      <input id="payment-expiry-year" type="text" class="field-input h-11 w-full" autocomplete="off" />

      <label class="field-label" for="payment-birth-business">birth_or_business_ev</label>
      <input id="payment-birth-business" type="text" class="field-input h-11 w-full" autocomplete="off" />

      <label class="field-label" for="payment-card-password">card_password_two_digits_ev</label>
      <input id="payment-card-password" type="text" class="field-input h-11 w-full" autocomplete="off" />

      <label class="field-label" for="payment-card-last4">card_last4 (required)</label>
      <input id="payment-card-last4" type="text" maxlength="4" class="field-input h-11 w-full" autocomplete="off" />

      <label class="field-label" for="payment-card-brand">card_brand (optional)</label>
      <input id="payment-card-brand" type="text" class="field-input h-11 w-full" autocomplete="off" />

      <div class="grid grid-cols-2 gap-2" data-action-group="pair">
        <button id="payment-modal-cancel" type="button" class="btn-ghost h-11 w-full">Cancel</button>
        <button id="payment-modal-save" type="submit" class="btn-primary h-11 w-full">Save</button>
      </div>
    </form>
  </div>
</div>
<script>
(() => {
  const statusNode = document.getElementById('payment-status');
  const form = document.getElementById('payment-form-universal');
  const cardListNode = document.getElementById('payment-card-list');
  const paymentModalOpenButton = document.getElementById('payment-modal-open');
  const paymentModal = document.getElementById('payment-modal');
  const paymentModalDeleteButton = document.getElementById('payment-modal-delete');
  const paymentModalCancelButton = document.getElementById('payment-modal-cancel');
  const paymentModalSelectedCard = document.getElementById('payment-modal-selected-card');
  const paymentPanInput = document.getElementById('payment-pan');
  const paymentExpiryMonthInput = document.getElementById('payment-expiry-month');
  const paymentExpiryYearInput = document.getElementById('payment-expiry-year');
  const paymentBirthBusinessInput = document.getElementById('payment-birth-business');
  const paymentCardPasswordInput = document.getElementById('payment-card-password');
  const paymentCardLast4Input = document.getElementById('payment-card-last4');
  const paymentCardBrandInput = document.getElementById('payment-card-brand');
  const MODAL_LAYER_BASE = 70;
  let modalLayerCounter = 0;
  let selectedPaymentMethodRef = null;
  let selectedPaymentLast4 = null;
  const currentThemeMode = () => document.body?.dataset?.themeMode === 'dark' ? 'dark' : 'light';
  const syncThemedSvgzIcons = () => {
    const mode = currentThemeMode();
    const icons = document.querySelectorAll('img[data-svgz-light][data-svgz-dark]');
    icons.forEach((icon) => {
      const nextSrc = mode === 'dark' ? icon.dataset.svgzDark : icon.dataset.svgzLight;
      if (!nextSrc || icon.getAttribute('src') === nextSrc) return;
      icon.setAttribute('src', nextSrc);
    });
  };

  const requestJson = async (url, method, payload) => {
    const response = await fetch(url, {
      method: method || 'GET',
      headers: { 'Content-Type': 'application/json', 'Accept': 'application/json' },
      body: payload ? JSON.stringify(payload) : undefined,
    });
    const text = await response.text();
    let body = null;
    try { body = text ? JSON.parse(text) : null; } catch (_err) {}
    return { ok: response.ok, status: response.status, body };
  };

  const escapeHtml = (value) => String(value || '')
    .replaceAll('&', '&amp;')
    .replaceAll('<', '&lt;')
    .replaceAll('>', '&gt;')
    .replaceAll('"', '&quot;')
    .replaceAll("'", '&#39;');

  const apiErrorMessage = (response, fallback) => {
    const body = response && response.body && typeof response.body === 'object' ? response.body : {};
    const message = typeof body.message === 'string' && body.message.trim()
      ? body.message.trim()
      : fallback;
    const requestId = typeof body.request_id === 'string' && body.request_id.trim()
      ? ` (request_id: ${body.request_id.trim()})`
      : '';
    return `${message}${requestId}`;
  };

  const showStatus = (kind, message) => {
    statusNode.classList.remove('hidden');
    statusNode.className = kind === 'error' ? 'error-card' : 'summary-card';
    statusNode.textContent = message;
  };

  const bringModalToBody = (modalNode) => {
    if (!modalNode || !document.body) return;
    if (modalNode.parentElement !== document.body) {
      document.body.appendChild(modalNode);
    }
  };

  const openModalLayer = (modalNode) => {
    if (!modalNode) return;
    bringModalToBody(modalNode);
    modalLayerCounter += 1;
    modalNode.style.zIndex = String(MODAL_LAYER_BASE + modalLayerCounter);
    modalNode.classList.remove('hidden');
    modalNode.setAttribute('aria-hidden', 'false');
  };

  const closeModalLayer = (modalNode) => {
    if (!modalNode) return;
    modalNode.classList.add('hidden');
    modalNode.setAttribute('aria-hidden', 'true');
    modalNode.style.removeProperty('z-index');
    if (!document.querySelector('.app-modal-backdrop:not(.hidden)')) {
      modalLayerCounter = 0;
    }
  };

  const resetPaymentForm = () => {
    if (form) form.reset();
  };

  const syncPaymentModalState = () => {
    const hasSelectedCard = !!(selectedPaymentMethodRef && selectedPaymentLast4);
    if (paymentModalDeleteButton) {
      paymentModalDeleteButton.disabled = !hasSelectedCard;
    }
    if (paymentModalSelectedCard) {
      paymentModalSelectedCard.textContent = hasSelectedCard
        ? `Selected card: •••• ${selectedPaymentLast4}`
        : 'Adding a new card';
    }
  };

  const openPaymentModal = () => {
    syncPaymentModalState();
    openModalLayer(paymentModal);
    requestAnimationFrame(() => {
      if (paymentPanInput) paymentPanInput.focus();
    });
  };

  const closePaymentModal = () => {
    closeModalLayer(paymentModal);
    resetPaymentForm();
  };

  const deleteSelectedCard = async () => {
    const paymentMethodRef = String(selectedPaymentMethodRef || '').trim();
    const last4 = String(selectedPaymentLast4 || '').trim();
    if (!paymentMethodRef) return;
    if (!window.confirm(`Delete saved card ending ${last4}?`)) return;

    const response = await requestJson(`/api/train/payment-methods/${encodeURIComponent(paymentMethodRef)}`, 'DELETE');
    if (!response.ok) {
      showStatus('error', apiErrorMessage(response, 'Could not delete saved card.'));
      return;
    }
    showStatus('success', `Deleted card ending ${last4}.`);
    selectedPaymentMethodRef = null;
    selectedPaymentLast4 = null;
    closePaymentModal();
    loadCards();
  };

  const renderCards = (cards) => {
    if (!Array.isArray(cards) || cards.length === 0) {
      cardListNode.innerHTML = '<div class="empty-card">No saved cards.</div>';
      return;
    }

    cardListNode.innerHTML = cards.map((card) => `
      <button type="button" class="summary-card w-full text-left" data-payment-ref="${escapeHtml(card.payment_method_ref || '')}" data-card-last4="${escapeHtml(card.card_last4 || '0000')}">
        <div class="summary-row">
          <span class="txt-strong">${escapeHtml((card.card_brand || 'Card').toUpperCase())} · •••• ${escapeHtml(card.card_last4 || '0000')}</span>
          <span class="text-xs txt-supporting">${escapeHtml(card.payment_method_ref || '')}</span>
        </div>
      </button>
    `).join('');

    Array.from(cardListNode.querySelectorAll('[data-payment-ref]')).forEach((button) => {
      button.addEventListener('click', async () => {
        selectedPaymentMethodRef = String(button.getAttribute('data-payment-ref') || '').trim() || null;
        selectedPaymentLast4 = String(button.getAttribute('data-card-last4') || '').trim() || null;
        openPaymentModal();
      });
    });
  };

  const loadCards = async () => {
    const response = await requestJson('/api/train/payment-methods');
    if (!response.ok) {
      cardListNode.innerHTML = `<div class="error-card">${escapeHtml(apiErrorMessage(response, 'Could not load saved cards.'))}</div>`;
      return;
    }
    const cards = response.body && Array.isArray(response.body.cards) ? response.body.cards : [];
    renderCards(cards);
  };

  if (paymentModalOpenButton) {
    paymentModalOpenButton.addEventListener('click', () => {
      selectedPaymentMethodRef = null;
      selectedPaymentLast4 = null;
      openPaymentModal();
    });
  }

  if (paymentModalCancelButton) {
    paymentModalCancelButton.addEventListener('click', () => {
      closePaymentModal();
    });
  }

  if (paymentModalDeleteButton) {
    paymentModalDeleteButton.addEventListener('click', async () => {
      await deleteSelectedCard();
    });
  }

  if (paymentModal) {
    paymentModal.addEventListener('click', (event) => {
      if (event.target === paymentModal) {
        closePaymentModal();
      }
    });
  }

  document.addEventListener('keydown', (event) => {
    if (event.key !== 'Escape') return;
    if (!paymentModal || paymentModal.classList.contains('hidden')) return;
    closePaymentModal();
  });

  form.addEventListener('submit', async (event) => {
    event.preventDefault();
    const response = await requestJson('/api/train/payment-methods', 'PUT', {
      pan_ev: String(paymentPanInput && paymentPanInput.value ? paymentPanInput.value : '').trim(),
      expiry_month_ev: String(paymentExpiryMonthInput && paymentExpiryMonthInput.value ? paymentExpiryMonthInput.value : '').trim(),
      expiry_year_ev: String(paymentExpiryYearInput && paymentExpiryYearInput.value ? paymentExpiryYearInput.value : '').trim(),
      birth_or_business_ev: String(paymentBirthBusinessInput && paymentBirthBusinessInput.value ? paymentBirthBusinessInput.value : '').trim(),
      card_password_two_digits_ev: String(paymentCardPasswordInput && paymentCardPasswordInput.value ? paymentCardPasswordInput.value : '').trim(),
      card_last4: String(paymentCardLast4Input && paymentCardLast4Input.value ? paymentCardLast4Input.value : '').trim(),
      card_brand: String(paymentCardBrandInput && paymentCardBrandInput.value ? paymentCardBrandInput.value : '').trim() || null,
    });
    if (!response.ok) {
      showStatus('error', apiErrorMessage(response, 'Could not save payment card.'));
      return;
    }
    closePaymentModal();
    showStatus('success', 'Payment card saved.');
    loadCards();
  });

  syncThemedSvgzIcons();
  if (document.body && typeof MutationObserver === 'function') {
    const themeObserver = new MutationObserver(() => syncThemedSvgzIcons());
    themeObserver.observe(document.body, { attributes: true, attributeFilter: ['data-theme-mode'] });
  }
  loadCards();
})();
</script>"#,
    );
    html.push_str(&bottom_nav);
    html
}

#[derive(Debug, Clone)]
pub struct AdminMaintenanceView {
    pub admin_email: String,
    pub db_ok: bool,
    pub redis_ok: bool,
    pub ready_ok: bool,
    pub health_path: &'static str,
    pub ready_path: &'static str,
    pub metrics_path: &'static str,
    pub metrics_snapshot: String,
}

pub fn render_admin_maintenance(view: &AdminMaintenanceView) -> String {
    let admin_email = html_escape(&view.admin_email);
    let metrics_snapshot = html_escape(&view.metrics_snapshot);
    let readiness = if view.ready_ok { "Healthy" } else { "Degraded" };
    format!(
        r#"<main class="mx-auto w-full max-w-[480px] px-4 pb-24 pt-4 md:max-w-7xl md:px-6 md:pt-8">
  <div class="admin-shell-grid">
    {}
    <div class="space-y-4">
      <section class="glass-card rounded-[22px] p-5">
        <p class="eyebrow">ops.bominal.com</p>
        <h1 class="mt-1 text-2xl font-semibold txt-strong">Admin maintenance</h1>
        <p class="mt-2 text-sm txt-supporting">Signed in as {admin_email}</p>
        <div class="mt-4 grid grid-cols-1 gap-2 lg:grid-cols-2">
          <div class="summary-row"><span>Readiness</span><span class="badge">{readiness}</span></div>
          <div class="summary-row"><span>Liveness</span><span class="badge">Healthy</span></div>
          <div class="summary-row"><span>Database</span><span class="badge">{}</span></div>
          <div class="summary-row"><span>Redis</span><span class="badge">{}</span></div>
        </div>
        <div class="action-group action-pair" data-action-group="pair">
          <a class="btn-ghost h-12 w-full text-center" href="{}">/health</a>
          <a class="btn-ghost h-12 w-full text-center" href="{}">/ready</a>
        </div>
        <div class="action-group action-pair" data-action-group="pair">
          <a class="btn-ghost h-12 w-full text-center" href="{}">metrics text</a>
          <a class="btn-primary h-12 w-full text-center" href="/admin/observability">Open observability</a>
        </div>
      </section>
      <section class="glass-card rounded-[22px] p-5">
        <h2 class="text-lg font-semibold txt-strong">Operations modules</h2>
        <div class="mt-3 grid grid-cols-1 gap-2 md:grid-cols-2">
          <a class="summary-card hover:border-indigo-300" href="/admin/users">
            <p class="txt-strong">Users and sessions</p>
            <p class="mt-1 text-xs txt-supporting">Roles, access toggles, and session revocation</p>
          </a>
          <a class="summary-card hover:border-indigo-300" href="/admin/runtime">
            <p class="txt-strong">Runtime operations</p>
            <p class="mt-1 text-xs txt-supporting">Retry, requeue, cancel, and kill switches</p>
          </a>
          <a class="summary-card hover:border-indigo-300" href="/admin/security">
            <p class="txt-strong">Security controls</p>
            <p class="mt-1 text-xs txt-supporting">Step-up policy and access safety checks</p>
          </a>
          <a class="summary-card hover:border-indigo-300" href="/admin/config">
            <p class="txt-strong">Redacted config</p>
            <p class="mt-1 text-xs txt-supporting">Safe visibility into runtime configuration state</p>
          </a>
        </div>
      </section>
      <section class="glass-card rounded-[22px] p-5">
        <h2 class="text-lg font-semibold txt-strong">Metrics snapshot</h2>
        <pre class="mt-3 max-h-[28rem] overflow-auto rounded-2xl bg-slate-950/90 p-4 text-xs txt-inverse">{metrics_snapshot}</pre>
      </section>
    </div>
  </div>
</main>
{}"#,
        admin_desktop_sidebar("maintenance"),
        if view.db_ok { "Healthy" } else { "Degraded" },
        if view.redis_ok { "Healthy" } else { "Degraded" },
        view.health_path,
        view.ready_path,
        view.metrics_path,
        admin_bottom_nav("maintenance"),
    )
}

pub fn render_admin_section(admin_email: &str, section: &str) -> String {
    let (title, subtitle) = match section {
        "users" => (
            "Users and sessions",
            "Review identities, roles, active sessions, and access state.",
        ),
        "runtime" => (
            "Runtime operations",
            "Control job lifecycle, queue recovery, and runtime kill switches.",
        ),
        "observability" => (
            "Observability",
            "Track health/readiness, incidents, and operational timelines.",
        ),
        "security" => (
            "Security controls",
            "Step-up enforcement, session posture, and privileged access policy.",
        ),
        "config" => (
            "Redacted config",
            "Visibility into safe configuration keys with secret protection.",
        ),
        "audit" => (
            "Audit log",
            "Immutable trail of privileged actions with request-level traceability.",
        ),
        _ => ("Admin", "Admin module"),
    };
    let mut html = format!(
        r#"<main data-admin-section="{}" class="mx-auto w-full max-w-[480px] px-4 pb-24 pt-4 md:max-w-7xl md:px-6 md:pt-8">
  <div class="admin-shell-grid">
    {}
    <div class="space-y-4">
      <section class="glass-card rounded-[22px] p-5">
        <p class="eyebrow">ops.bominal.com</p>
        <h1 class="mt-1 text-2xl font-semibold txt-strong">{}</h1>
        <p class="mt-2 text-sm txt-supporting">Operator: {}</p>
        <p class="mt-1 text-sm txt-supporting">{}</p>
        <div id="admin-flash" class="mt-3 hidden"></div>
        <div class="mt-3 flex flex-wrap gap-2 md:hidden">
          <a class="btn-chip" href="/admin/security">Security</a>
          <a class="btn-chip" href="/admin/config">Config</a>
          <a class="btn-chip" href="/admin/audit">Audit</a>
        </div>
      </section>
      <section class="glass-card rounded-[22px] p-5">
        <div id="admin-content" class="space-y-2"><div class="loading-card">Loading...</div></div>
      </section>
    </div>
  </div>
</main>
{}"#,
        section,
        admin_desktop_sidebar(section),
        title,
        html_escape(admin_email),
        subtitle,
        admin_bottom_nav(section),
    );
    html.push_str(ADMIN_CONFIRM_MODAL);
    html.push_str(ADMIN_SECTION_SCRIPT);
    html
}

fn admin_desktop_sidebar(active: &str) -> String {
    let section = match active {
        "maintenance" => AdminSection::Maintenance,
        "users" => AdminSection::Users,
        "runtime" => AdminSection::Runtime,
        "observability" => AdminSection::Observability,
        "security" => AdminSection::Security,
        "config" => AdminSection::Config,
        "audit" => AdminSection::Audit,
        _ => AdminSection::Maintenance,
    };
    render_admin_sidebar(section)
}

fn admin_bottom_nav(active: &str) -> String {
    let section = match active {
        "maintenance" => AdminSection::Maintenance,
        "users" => AdminSection::Users,
        "runtime" => AdminSection::Runtime,
        "observability" => AdminSection::Observability,
        "audit" => AdminSection::Audit,
        _ => AdminSection::Maintenance,
    };
    render_admin_bottom_nav(section)
}

const ADMIN_CONFIRM_MODAL: &str = r##"
<div id="admin-confirm-modal" class="app-modal-backdrop hidden" role="dialog" aria-modal="true" aria-labelledby="admin-confirm-title">
  <div class="app-modal-card">
    <h3 id="admin-confirm-title" class="text-base font-semibold txt-strong">Confirm action</h3>
    <p id="admin-confirm-message" class="mt-2 text-sm txt-supporting"></p>
    <label class="field-label mt-3" for="admin-confirm-target">Type target value</label>
    <input id="admin-confirm-target" class="field-input h-11 w-full" autocomplete="off" />
    <label class="field-label mt-3" for="admin-confirm-reason">Reason for change</label>
    <textarea id="admin-confirm-reason" class="field-input min-h-[96px] w-full py-3" maxlength="500"></textarea>
    <p class="mt-2 text-xs txt-faint">This action is audited and may require recent step-up authentication.</p>
    <div class="action-group action-pair" data-action-group="pair">
      <button id="admin-confirm-cancel" type="button" class="btn-ghost h-11 w-full">Cancel</button>
      <button id="admin-confirm-submit" type="button" class="btn-primary h-11 w-full">Confirm</button>
    </div>
  </div>
</div>
"##;

const ADMIN_SECTION_SCRIPT: &str =
    r#"<script type="module" src="/assets/js/admin/entry.js"></script>"#;

#[cfg(test)]
mod tests {
    use super::*;

    fn index_of(haystack: &str, needle: &str) -> usize {
        haystack.find(needle).unwrap_or_else(|| {
            panic!("expected substring not found: {needle}");
        })
    }

    #[test]
    fn auth_landing_keeps_passkey_first_ordering() {
        let html = render_auth_landing();
        let passkey_index = index_of(&html, "id=\"passkey-primary\"");
        let email_index = index_of(&html, "id=\"toggle-email\"");
        assert!(passkey_index < email_index);
        assert!(html.contains("data-action-group=\"pair\""));
    }

    #[test]
    fn auth_email_form_uses_secondary_then_primary_actions() {
        let html = render_auth_landing();
        let back_index = index_of(&html, "id=\"back-passkey\"");
        let continue_index = index_of(&html, "id=\"email-continue\"");
        assert!(back_index < continue_index);
    }

    #[test]
    fn dashboard_pages_use_svgz_icon_assets() {
        let auth_html = render_auth_landing();
        assert!(auth_html.contains("/assets/icons/runtime-ui/auth-hero-passkey-light.svgz"));
        assert!(auth_html.contains("/assets/icons/runtime-ui/auth-hero-password-light.svgz"));
        assert!(auth_html.contains("/assets/icons/runtime-ui/theme-mini-sun-active.svgz"));
        assert!(auth_html.contains("/assets/icons/runtime-ui/theme-mini-moon-active.svgz"));
        assert!(!auth_html.contains("<svg id=\"auth-hero-passkey-icon\""));
        assert!(!auth_html.contains("<svg id=\"auth-hero-password-icon\""));

        let settings_html = render_dashboard_settings("admin@bominal.local");
        assert!(settings_html.contains("/assets/icons/runtime-ui/icon-plus-btn-chip-light.svgz"));
        assert!(
            settings_html.contains("/assets/icons/runtime-ui/icon-trash-destructive-light.svgz")
        );
        assert!(!settings_html.contains("<svg class=\"h-4 w-4\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"1.8\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\">"));

        let providers_html = render_dashboard_settings_providers("admin@bominal.local");
        assert!(
            providers_html.contains("/assets/icons/runtime-ui/icon-trash-destructive-light.svgz")
        );
        assert!(providers_html.contains("provider-status-credentials"));
        assert!(providers_html.contains("provider-status-payment"));
        assert!(providers_html.contains("${iconPrefix}-green.svgz"));

        let payment_html = render_dashboard_payment("admin@bominal.local");
        assert!(payment_html.contains("/assets/icons/runtime-ui/icon-plus-btn-chip-light.svgz"));
        assert!(
            payment_html.contains("/assets/icons/runtime-ui/icon-trash-destructive-light.svgz")
        );

        let train_html = render_dashboard_train("admin@bominal.local");
        assert!(train_html.contains("/assets/icons/runtime-ui/icon-search-supporting-light.svgz"));
        let train_js = include_str!("../../../../frontend/assets/js/dashboard/train.js");
        assert!(train_js.contains("provider-status-payment"));
        assert!(train_js.contains("provider-status-credentials"));
        assert!(train_js.contains("${iconPrefix}-gray-dark.svgz"));
    }

    #[test]
    fn job_detail_uses_sticky_action_group_with_secondary_then_primary() {
        let html = render_dashboard_job_detail("admin@bominal.local", "job-123");
        let sticky_index = index_of(
            &html,
            "class=\"action-sticky\" data-action-group=\"sticky\"",
        );
        let sticky_block = &html[sticky_index..];
        let back_index = index_of(sticky_block, "href=\"/dashboard/jobs\"");
        let refresh_index = index_of(sticky_block, "id=\"manual-refresh\"");
        assert!(back_index < refresh_index);
    }

    #[test]
    fn passkey_delete_action_is_destructive_and_prompted_in_modal() {
        let html = render_dashboard_settings("admin@bominal.local");
        assert!(html.contains("id=\"passkey-modal-delete\""));
        assert!(html.contains("class=\"btn-destructive h-9 w-9 p-0\""));
        assert!(html.contains("type=\"module\" src=\"/assets/js/dashboard/security.js\""));
        assert!(!html.contains("const modalResult = await openSecurityModal({"));
        assert!(html.contains("Settings tabs"));
        assert!(html.contains("href=\"/dashboard/settings\""));
        assert!(html.contains("href=\"/dashboard/settings/providers\""));
        assert!(html.contains("href=\"/dashboard/payment\""));
        assert!(html.contains(">Account</a>"));
        assert!(html.contains(">Providers</a>"));
        assert!(html.contains(">Payment</a>"));
    }

    #[test]
    fn dashboard_pages_load_external_module_scripts() {
        let overview = render_dashboard_overview("admin@bominal.local");
        assert!(overview.contains("type=\"module\" src=\"/assets/js/dashboard/overview.js\""));

        let jobs = render_dashboard_jobs("admin@bominal.local");
        assert!(jobs.contains("type=\"module\" src=\"/assets/js/dashboard/jobs.js\""));

        let job_detail = render_dashboard_job_detail("admin@bominal.local", "job-123");
        assert!(job_detail.contains("type=\"module\" src=\"/assets/js/dashboard/job-detail.js\""));

        let settings = render_dashboard_settings("admin@bominal.local");
        assert!(settings.contains("type=\"module\" src=\"/assets/js/dashboard/security.js\""));

        let train = render_dashboard_train("admin@bominal.local");
        assert!(train.contains("type=\"module\" src=\"/assets/js/dashboard/train.js\""));
    }

    #[test]
    fn admin_section_loads_external_module_script() {
        let html = render_admin_section("ops@bominal.com", "observability");
        assert!(html.contains("type=\"module\" src=\"/assets/js/admin/entry.js\""));
    }

    #[test]
    fn admin_runtime_section_preserves_runtime_mount_point() {
        let html = render_admin_section("ops@bominal.com", "runtime");
        assert!(html.contains("data-admin-section=\"runtime\""));
        assert!(html.contains("id=\"admin-content\""));
    }

    #[test]
    fn dashboard_train_page_uses_session_train_api_contract() {
        let html = render_dashboard_train("admin@bominal.local");
        let train_js = include_str!("../../../../frontend/assets/js/dashboard/train.js");

        assert!(html.contains("type=\"module\" src=\"/assets/js/dashboard/train.js\""));
        assert!(html.contains("id=\"station-picker-correction\""));
        assert!(html.contains("data-i18n=\"search.title\""));
        assert!(html.contains("data-i18n-placeholder=\"station.search_placeholder\""));
        assert!(html.contains("station-picker-modal"));
        assert!(html.contains("passenger-picker-modal"));
        assert!(!html.contains("/dashboard/settings/providers"));

        assert!(train_js.contains("/api/train/preflight"));
        assert!(train_js.contains("/api/train/stations/regions"));
        assert!(train_js.contains("/api/train/stations/suggest"));
        assert!(train_js.contains("/api/train/providers/all/search"));
        assert!(train_js.contains("/api/train/tasks"));
        assert!(train_js.contains("/api/train/tasks/${encodeURIComponent(taskId)}"));
        assert!(train_js.contains("const TRAIN_I18N = {"));
        assert!(train_js.contains("'station.correction_prompt'"));
        assert!(train_js.contains("params.set('apply_mode', 'suggest')"));
        assert!(train_js.contains("params.set('lang_hint', suggestLangHint())"));
        assert!(train_js.contains("params.set('layout_hint', suggestLayoutHint(query))"));
        assert!(train_js.contains("corrected_query"));
        assert!(train_js.contains("autocorrect_applied"));
        assert!(
            train_js.contains(
                "document.body?.dataset?.locale || document.documentElement?.lang || 'en'"
            )
        );
        assert!(!train_js.contains("const normalizeLocale ="));
    }

    #[test]
    fn dashboard_provider_security_page_posts_to_provider_credentials_route() {
        let html = render_dashboard_settings_providers("admin@bominal.local");
        assert!(html.contains("/api/train/providers/${submittedProvider}/credentials"));
        assert!(html.contains("/api/train/providers/${deletedProvider}/credentials"));
        assert!(html.contains("data-provider-open"));
        assert!(html.contains("provider-auth-modal"));
        assert!(html.contains("provider-auth-form"));
        assert!(html.contains("provider-auth-modal-delete"));
        assert!(!html.contains("data-provider-edit=\""));
        assert!(!html.contains("data-provider-delete=\""));
        assert!(!html.contains("data-provider-reauth"));
        assert!(!html.contains("Sign out / Sign in"));
        assert!(!html.contains("provider-action-edit"));
        assert!(!html.contains("provider-action-delete"));
        assert!(!html.contains("provider-form-srt"));
        assert!(!html.contains("provider-form-ktx"));
        assert!(!html.contains("provider-editor-srt"));
        assert!(!html.contains("provider-editor-ktx"));
        assert!(!html.contains("data-provider-editor-body"));
        assert!(html.contains("Settings tabs"));
        assert!(html.contains("href=\"/dashboard/settings\""));
        assert!(html.contains("href=\"/dashboard/settings/providers\""));
        assert!(html.contains("href=\"/dashboard/payment\""));
        assert!(html.contains(">Account</a>"));
        assert!(html.contains(">Providers</a>"));
        assert!(html.contains(">Payment</a>"));
    }

    #[test]
    fn dashboard_provider_security_delete_feedback_uses_deleted_provider_snapshot() {
        let html = render_dashboard_settings_providers("admin@bominal.local");
        assert!(html.contains("const deletedProvider = activeProvider;"));
        assert!(html.contains("${deletedProvider.toUpperCase()}: Credentials removed."));
    }

    #[test]
    fn dashboard_provider_security_submit_shows_auth_probe_status_feedback() {
        let html = render_dashboard_settings_providers("admin@bominal.local");
        assert!(html.contains("const providerAuthProbeStatus = (provider) => {"));
        assert!(html.contains("const hasError = probeStatus === 'error';"));
        assert!(html.contains("const probeStatus = typeof result.auth_probe_status === 'string'"));
        assert!(html.contains(
            "showStatus('error', probeMessage || `${submittedProvider.toUpperCase()}: Authentication failed.`);"
        ));
        assert!(html.contains(
            "showStatus('success', probeMessage || `${submittedProvider.toUpperCase()}: Successfully authenticated.`);"
        ));
    }

    #[test]
    fn dashboard_payment_page_uses_universal_payment_routes() {
        let html = render_dashboard_payment("admin@bominal.local");
        assert!(html.contains("/api/train/payment-methods"));
        assert!(
            html.contains("/api/train/payment-methods/${encodeURIComponent(paymentMethodRef)}")
        );
        assert!(html.contains("payment-form-universal"));
        assert!(html.contains("payment-modal-open"));
        assert!(html.contains("payment-modal-delete"));
        assert!(!html.contains("Save up to 3 cards shared across SRT and KTX"));
        assert!(html.contains("Settings tabs"));
        assert!(html.contains("href=\"/dashboard/settings\""));
        assert!(html.contains("href=\"/dashboard/settings/providers\""));
        assert!(html.contains("href=\"/dashboard/payment\""));
        assert!(html.contains(">Account</a>"));
        assert!(html.contains(">Providers</a>"));
        assert!(html.contains(">Payment</a>"));
    }
}
