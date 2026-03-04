const MODAL_LAYER_BASE = 70;

export const createConfirmModal = () => {
  const modal = document.getElementById("admin-confirm-modal");
  const modalTitle = document.getElementById("admin-confirm-title");
  const modalMessage = document.getElementById("admin-confirm-message");
  const modalTarget = document.getElementById("admin-confirm-target");
  const modalReason = document.getElementById("admin-confirm-reason");
  const modalCancel = document.getElementById("admin-confirm-cancel");
  const modalSubmit = document.getElementById("admin-confirm-submit");

  if (
    !modal ||
    !modalTitle ||
    !modalMessage ||
    !modalTarget ||
    !modalReason ||
    !modalCancel ||
    !modalSubmit
  ) {
    return {
      available: false,
      open: async () => null,
      teardown: () => {},
    };
  }

  let modalResolver = null;
  let modalLayerCounter = 0;

  const bringModalToBody = () => {
    if (modal.parentElement !== document.body) {
      document.body.appendChild(modal);
    }
  };

  const close = (value) => {
    modal.classList.add("hidden");
    modal.style.removeProperty("z-index");
    if (!document.querySelector(".app-modal-backdrop:not(.hidden)")) {
      modalLayerCounter = 0;
    }
    if (modalResolver) {
      modalResolver(value);
      modalResolver = null;
    }
  };

  const onCancelClick = () => close(null);
  const onSubmitClick = () => {
    const payload = {
      confirm_target: modalTarget.value.trim(),
      reason: modalReason.value.trim(),
    };
    if (!payload.confirm_target || !payload.reason) {
      return;
    }
    close(payload);
  };
  const onBackdropClick = (event) => {
    if (event.target === modal) close(null);
  };
  const onKeyDown = (event) => {
    if (event.key === "Escape" && !modal.classList.contains("hidden")) {
      close(null);
    }
  };

  modalCancel.addEventListener("click", onCancelClick);
  modalSubmit.addEventListener("click", onSubmitClick);
  modal.addEventListener("click", onBackdropClick);
  document.addEventListener("keydown", onKeyDown);

  return {
    available: true,
    open: ({ title, message, targetLabel, confirmText }) =>
      new Promise((resolve) => {
        bringModalToBody();
        modalLayerCounter += 1;
        modal.style.zIndex = String(MODAL_LAYER_BASE + modalLayerCounter);
        modalResolver = resolve;
        modalTitle.textContent = title || "Confirm action";
        modalMessage.textContent = message || "";
        modalTarget.value = "";
        modalTarget.placeholder = targetLabel || "";
        modalReason.value = "";
        modalSubmit.textContent = confirmText || "Confirm";
        modal.classList.remove("hidden");
        window.setTimeout(() => modalTarget.focus(), 0);
      }),
    teardown: () => {
      modalCancel.removeEventListener("click", onCancelClick);
      modalSubmit.removeEventListener("click", onSubmitClick);
      modal.removeEventListener("click", onBackdropClick);
      document.removeEventListener("keydown", onKeyDown);
    },
  };
};
