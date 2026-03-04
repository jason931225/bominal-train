fn html_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn app_shell_topbar(title: &str, subtitle: &str) -> String {
    format!(
        r#"<header class="mx-auto w-full max-w-[480px] px-4 pt-4 md:max-w-7xl md:px-6">
  <div class="glass-card rounded-[20px] p-4">
    <div class="flex items-start justify-between gap-3">
      <div>
        <p class="eyebrow">bominal</p>
        <h1 class="mt-1 text-xl font-semibold txt-strong">{title}</h1>
      </div>
      <button
        type="button"
        class="theme-mini-switch theme-inline-switch"
        data-theme-toggle
        data-theme-toggle-compact
        aria-label="Theme toggle"
      >
        <svg class="theme-mini-icon theme-mini-icon-sun" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
          <circle cx="12" cy="12" r="4"></circle>
          <line x1="12" y1="2" x2="12" y2="4.5"></line>
          <line x1="12" y1="19.5" x2="12" y2="22"></line>
          <line x1="4.9" y1="4.9" x2="6.7" y2="6.7"></line>
          <line x1="17.3" y1="17.3" x2="19.1" y2="19.1"></line>
          <line x1="2" y1="12" x2="4.5" y2="12"></line>
          <line x1="19.5" y1="12" x2="22" y2="12"></line>
          <line x1="4.9" y1="19.1" x2="6.7" y2="17.3"></line>
          <line x1="17.3" y1="6.7" x2="19.1" y2="4.9"></line>
        </svg>
        <svg class="theme-mini-icon theme-mini-icon-moon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
          <path d="M21 12.8A8.5 8.5 0 1 1 11.2 3a6.8 6.8 0 0 0 9.8 9.8z"></path>
        </svg>
        <span class="theme-mini-thumb" aria-hidden="true"></span>
      </button>
    </div>
    <p class="mt-1 text-sm txt-supporting">{subtitle}</p>
  </div>
</header>"#
    )
}

fn dashboard_desktop_sidebar(active: &str) -> String {
    format!(
        r#"<aside class="hidden md:sticky md:top-6 md:block md:self-start">
  <div class="glass-card rounded-[22px] p-3">
    <p class="eyebrow px-3 pt-1">navigation</p>
    <nav class="mt-2 space-y-1">
      <a href="/dashboard" class="desktop-side-link {}">Overview</a>
      <a href="/dashboard/jobs" class="desktop-side-link {}">Jobs</a>
      <a href="/dashboard/security" class="desktop-side-link {}">Security</a>
    </nav>
  </div>
</aside>"#,
        if active == "home" { "active" } else { "" },
        if active == "jobs" { "active" } else { "" },
        if active == "security" { "active" } else { "" },
    )
}

pub fn render_auth_landing() -> String {
    let html = r#"
<main class="mx-auto flex min-h-[100dvh] w-full px-4 py-6 2xl:px-8">
  <div class="my-auto mx-auto w-full 2xl:grid 2xl:max-w-[1600px] 2xl:grid-cols-[3fr_2fr] 2xl:items-center 2xl:gap-12">
    <section class="glass-card hidden rounded-[22px] p-6 md:p-8 2xl:block">
      <p class="eyebrow">dashboard preview</p>
      <h2 class="auth-title mt-2 text-2xl font-semibold">Operational clarity at a glance</h2>
      <p class="auth-copy mt-2 text-sm">Track runtime health, active sessions, and high-priority jobs in one place.</p>
      <div class="mt-5 space-y-2">
        <div class="summary-row"><span>Queued jobs</span><span class="badge">24</span></div>
        <div class="summary-row"><span>Running jobs</span><span class="badge">6</span></div>
        <div class="summary-row"><span>Error rate (5m)</span><span>0.3%</span></div>
        <div class="summary-row"><span>P95 latency</span><span>182ms</span></div>
      </div>
    </section>

    <div class="relative mb-8 mx-auto w-[90%] max-w-[420px] 2xl:mb-0 2xl:w-full 2xl:max-w-[420px] 2xl:justify-self-center">
      <section class="glass-card rounded-[22px] p-6 md:p-8">
        <p class="eyebrow">bominal authentication</p>
        <h1 class="sr-only" aria-label="Sign in securely"></h1>
        <div class="mt-3 flex justify-center">
          <div class="auth-hero-icon" role="img" aria-label="Secure account sign-in">
            <svg class="auth-hero-icon-main" viewBox="0 0 512 256" aria-hidden="true">
              <g transform="scale(10.6666667)" fill="none" stroke="rgb(var(--text-supporting))" stroke-width="1.125" stroke-linecap="round" stroke-linejoin="round">
                <path d="M7 2.5H4.5C3.4 2.5 2.5 3.4 2.5 4.5V7"></path>
                <path d="M17 2.5H19.5C20.6 2.5 21.5 3.4 21.5 4.5V7"></path>
                <path d="M2.5 17V19.5C2.5 20.6 3.4 21.5 4.5 21.5H7"></path>
                <path d="M17 21.5H19.5C20.6 21.5 21.5 20.6 21.5 19.5V17"></path>
                <path d="M9 8.5V9.5"></path>
                <path d="M15 8.5V9.5"></path>
                <path d="M12 8.5V12.3c0 .8-.7 1.5-1.5 1.5"></path>
                <path d="M8.9 15.2c.8 1.2 1.9 1.8 3.1 1.8s2.3-.6 3.1-1.8"></path>
              </g>
              <g transform="translate(256 0) scale(10.6666667)" fill="rgb(var(--text-supporting))">
                <g fill="none" stroke="rgb(var(--text-supporting))" stroke-width="1.125" stroke-linecap="round" stroke-linejoin="round">
                  <path d="M7 2.5H4.5C3.4 2.5 2.5 3.4 2.5 4.5V7"></path>
                  <path d="M17 2.5H19.5C20.6 2.5 21.5 3.4 21.5 4.5V7"></path>
                  <path d="M2.5 17V19.5C2.5 20.6 3.4 21.5 4.5 21.5H7"></path>
                  <path d="M17 21.5H19.5C20.6 21.5 21.5 20.6 21.5 19.5V17"></path>
                </g>
                <g transform="translate(12 12) scale(0.5) translate(-12 -12)">
                <path d="M18.4558 14.6734C18.4921 15.5205 18.3742 16.4404 18.2653 17.3604C18.2562 17.4514 18.238 17.5425 18.2108 17.6245C18.1564 17.8613 18.0384 18.0708 17.7571 18.0252C17.4759 17.9706 17.4123 17.7429 17.4577 17.497C17.5484 16.8958 17.5938 16.2947 17.6483 15.6936C17.8025 13.9539 17.6483 12.2325 17.2218 10.5474C16.5322 7.84233 13.801 6.12999 11.0879 6.62183C8.28416 7.13189 6.33331 9.63663 6.58737 12.4419C6.65996 13.2435 6.88681 14.0268 6.83237 14.8374C6.82329 15.0013 6.82329 15.1744 6.787 15.3383C6.74163 15.5569 6.63274 15.7573 6.3696 15.73C6.08832 15.7027 5.98851 15.4841 6.01573 15.2382C6.05202 14.8465 6.07925 14.4548 6.0248 14.0632C5.8887 13.0158 5.68908 11.9683 5.8887 10.9027C6.4422 7.96073 9.00099 5.77477 11.9681 5.75656C14.8898 5.73834 17.5121 7.89698 18.0838 10.8025C18.3288 12.0503 18.483 13.3163 18.4558 14.6734Z"></path>
                <path d="M21.0963 11.112C21.1053 11.4399 21.1235 11.7132 20.7877 11.7496C20.4702 11.786 20.3885 11.531 20.3431 11.276C20.1344 9.84601 19.7443 8.47978 18.8913 7.29572C16.7772 4.35379 13.9099 3.19705 10.3802 3.78908C7.16809 4.32646 4.55485 6.95872 3.8471 10.183C3.78359 10.4745 3.79266 10.9208 3.3299 10.8206C2.83992 10.7204 3.03047 10.3014 3.10306 9.99174C3.99228 6.10255 7.24975 3.26992 11.1242 2.88737C15.7337 2.43197 20.1617 5.63804 20.9874 10.5018C21.0327 10.7386 21.069 10.9663 21.0963 11.112Z"></path>
                <path d="M12.1405 2.22257C7.19532 2.04952 2.73105 5.82029 2.23199 11.2123C2.14126 12.2051 2.27736 13.1797 2.43161 14.1543C2.58587 15.0924 2.30458 15.9212 1.72386 16.659C1.56054 16.8685 1.35184 17.078 1.07963 16.8503C0.798344 16.6226 0.961671 16.3858 1.13407 16.1763C1.68757 15.5023 1.76016 14.7281 1.61498 13.8992C0.607796 7.85141 4.50042 2.44117 10.5617 1.50303C11.4418 1.3664 12.3401 1.32997 13.2293 1.4757C13.4834 1.52124 13.7375 1.585 13.7193 1.90378C13.6921 2.259 13.4199 2.24989 13.1477 2.24079C12.812 2.21346 12.4762 2.22257 12.1405 2.22257Z"></path>
                <path d="M14.1094 14.6278C14.064 17.9432 13.075 20.9398 11.0969 23.6085C10.9335 23.8271 10.7521 24.1003 10.4345 23.8726C10.1351 23.654 10.2802 23.399 10.4526 23.1622C11.9498 21.1402 12.8844 18.8813 13.2111 16.3766C13.4107 14.8373 13.3472 13.3254 13.0477 11.8134C12.9207 11.1667 12.3581 10.8297 11.723 10.9664C11.1876 11.0848 10.8519 11.6221 10.9698 12.2324C11.2511 13.7261 11.2874 15.2108 11.0152 16.7045C10.9608 17.0051 10.9608 17.4423 10.5162 17.3785C10.0625 17.3147 10.1986 16.914 10.253 16.6043C10.4889 15.229 10.498 13.8627 10.2258 12.4965C9.98988 11.3034 10.6069 10.3834 11.7502 10.1739C12.7392 9.99178 13.6466 10.7022 13.8553 11.8316C13.9188 12.1777 13.9823 12.5147 14.0095 12.8609C14.0549 13.4438 14.1457 14.0358 14.1094 14.6278Z"></path>
                <path d="M22.8111 14.6825C22.8564 15.484 22.7748 16.3402 22.684 17.1963C22.6568 17.4787 22.6477 17.8885 22.2122 17.8248C21.7948 17.761 21.8765 17.3876 21.9128 17.087C22.2122 14.5732 22.1033 12.0775 21.4863 9.63656C20.6424 6.25743 18.5192 3.98039 15.2799 2.73257C15.2073 2.70525 15.1347 2.68703 15.0621 2.65971C14.8443 2.56863 14.6719 2.432 14.7536 2.17698C14.8443 1.90373 15.0712 1.87641 15.3162 1.94927C15.9423 2.14054 16.5411 2.39557 17.1037 2.72347C20.2704 4.55421 22.0307 7.33219 22.5388 10.939C22.7112 12.1595 22.8473 13.38 22.8111 14.6825Z"></path>
                <path d="M11.8227 7.21389C12.0042 7.21389 12.0677 7.20478 12.1221 7.21389C12.3853 7.24121 12.6847 7.28675 12.6938 7.61465C12.7029 7.97897 12.3944 8.02451 12.104 8.02451C10.5796 7.99719 9.41817 8.63476 8.61061 9.91901C8.07526 10.7752 7.90286 11.7133 8.09341 12.7152C8.70135 16.0033 7.81212 18.8359 5.4711 21.204C5.38944 21.286 5.2987 21.3771 5.19889 21.4499C4.99927 21.6139 4.7815 21.6685 4.60002 21.4499C4.42762 21.2496 4.51836 21.0492 4.68169 20.8852C5.17167 20.4116 5.6435 19.9198 6.0246 19.3551C7.28585 17.4697 7.81212 15.4021 7.35844 13.1706C6.7505 10.1103 8.80116 7.3414 11.8227 7.21389Z"></path>
                <path d="M17.0128 14.3092C17.0037 17.7065 16.2415 20.6303 14.7171 23.3627C14.6899 23.4173 14.6627 23.472 14.6264 23.5175C14.4812 23.7543 14.2907 23.9001 14.0275 23.7361C13.7825 23.5904 13.8279 23.3536 13.9549 23.135C14.2635 22.5976 14.5357 22.042 14.7897 21.4682C15.7969 19.2003 16.2597 16.823 16.2415 14.3456C16.2324 13.2435 16.1145 12.1323 15.8604 11.0576C15.6517 10.1377 15.1618 9.39078 14.4086 8.81697C14.2544 8.69856 14.082 8.58926 13.9368 8.47086C13.7825 8.34334 13.6918 8.18851 13.8098 7.99723C13.9277 7.79686 14.1092 7.75131 14.3179 7.8515C14.826 8.10653 15.2434 8.46175 15.6154 8.89894C16.3504 9.76422 16.668 10.7934 16.8132 11.8955C16.9221 12.7881 17.04 13.6716 17.0128 14.3092Z"></path>
                <path d="M12.0223 0.0183811C13.7554 -0.100025 15.3796 0.309842 16.9493 1.01117C16.9947 1.02939 17.031 1.0476 17.0763 1.06582C17.3395 1.17512 17.5572 1.32996 17.4302 1.64874C17.2941 1.97664 17.0219 1.88555 16.7769 1.76715C15.924 1.36639 15.0348 1.0476 14.1092 0.910981C9.6994 0.218761 6.03362 1.56677 3.16632 5.02787C2.97577 5.26468 2.80337 5.67455 2.42227 5.39219C2.01396 5.09163 2.37691 4.79106 2.56745 4.54514C4.62719 1.96753 7.30394 0.464681 10.5705 0.0274892C11.0423 -0.0362679 11.5323 0.0183811 12.0223 0.0183811Z"></path>
                <path d="M19.9166 14.3454C19.9257 16.9595 19.5264 19.2456 18.7461 21.468C18.6916 21.6137 18.6372 21.7504 18.5827 21.8961C18.492 22.1329 18.3378 22.306 18.0655 22.2058C17.8024 22.1056 17.7752 21.887 17.8659 21.6411C17.9839 21.2949 18.1018 20.9579 18.2107 20.6027C18.8822 18.5078 19.2179 16.3492 19.1362 14.145C19.0637 12.287 18.9638 10.4107 18.0565 8.70747C17.9204 8.45244 17.757 8.21563 17.6119 7.9606C17.4939 7.75112 17.4757 7.54163 17.6844 7.3959C17.9113 7.23195 18.0837 7.35946 18.2289 7.54163C18.9003 8.43423 19.2996 9.44523 19.5173 10.52C19.7805 11.8771 19.9438 13.2251 19.9166 14.3454Z"></path>
                <path d="M4.61846 14.4365C4.57309 14.136 4.50958 13.6714 4.44606 13.2069C4.06496 10.4471 4.90882 8.12457 6.94133 6.23007C7.01392 6.15721 7.10466 6.09345 7.18632 6.02969C7.35872 5.9204 7.53112 5.92039 7.6763 6.07523C7.83056 6.23007 7.80334 6.41224 7.6763 6.56707C7.55835 6.7037 7.42224 6.83121 7.29521 6.95873C5.48954 8.74392 4.80901 10.8843 5.26269 13.3891C5.64379 15.4931 5.07215 17.3238 3.66572 18.8995C3.48425 19.0999 3.26648 19.273 3.04871 19.4369C2.88538 19.5553 2.70391 19.5644 2.55873 19.4005C2.40447 19.2183 2.44077 19.0361 2.59502 18.8722C2.73113 18.7265 2.87631 18.599 3.01241 18.4532C4.05589 17.3785 4.57309 16.0942 4.61846 14.4365Z"></path>
                <path d="M12.6577 14.6552C12.5942 17.8886 11.5688 20.7668 9.50003 23.2715C9.45466 23.3262 9.40929 23.3899 9.35485 23.4446C9.18245 23.6358 8.97376 23.7269 8.76506 23.5448C8.56544 23.3808 8.61081 23.1622 8.75599 22.971C9.00098 22.6431 9.26412 22.3334 9.50003 21.9964C11.0335 19.856 11.7957 17.4514 11.8955 14.8192C11.9227 13.9994 11.8138 13.1888 11.6868 12.3782C11.6687 12.2871 11.6505 12.196 11.6414 12.1049C11.6233 11.8681 11.6868 11.6677 11.9499 11.6313C12.2131 11.5858 12.3401 11.7497 12.3855 11.9865C12.567 12.87 12.6668 13.7626 12.6577 14.6552Z"></path>
                <path d="M23.9998 11.5857C23.9998 11.7679 23.9998 11.95 23.9998 12.1322C23.9998 12.369 23.9363 12.5694 23.655 12.5785C23.3737 12.5876 23.2648 12.3872 23.2467 12.1322C23.2285 11.6768 23.2195 11.2214 23.165 10.7751C22.7839 7.55078 21.2868 4.95495 18.7371 2.96937C18.5102 2.78721 18.1382 2.61415 18.3923 2.25894C18.6373 1.92193 18.9367 2.17696 19.1726 2.35913C22.2214 4.6726 23.773 7.77848 23.9998 11.5857Z"></path>
                <path d="M15.5522 14.0449C15.5522 14.1815 15.5522 14.3181 15.5522 14.4547C15.5431 14.7007 15.4615 14.9284 15.1711 14.9284C14.8807 14.9284 14.8082 14.7098 14.7991 14.4547C14.7537 13.5348 14.6811 12.6149 14.5178 11.7041C14.1821 9.86423 12.3583 8.92609 10.7794 9.79137C10.2532 10.0828 9.85391 10.52 9.64521 11.1029C9.61799 11.1758 9.59077 11.2487 9.57262 11.3215C9.49096 11.5766 9.39115 11.8498 9.05542 11.7496C8.69247 11.6403 8.76506 11.3306 8.84673 11.0665C9.20967 9.95532 9.9265 9.17201 11.0516 8.84412C12.9662 8.27941 14.8082 9.40883 15.2437 11.3944C15.4342 12.2597 15.5068 13.1523 15.5522 14.0449Z"></path>
                <path d="M9.73564 14.6279C9.69934 17.497 8.67401 19.9653 6.70502 22.0511C6.6415 22.1149 6.57799 22.1786 6.51447 22.2424C6.32392 22.4245 6.09708 22.5338 5.87931 22.3152C5.64339 22.0875 5.7795 21.8689 5.97005 21.6868C6.62335 21.0492 7.17685 20.3388 7.63961 19.5464C8.65587 17.7794 9.11863 15.8849 8.93715 13.8355C8.92808 13.6807 8.919 13.535 8.919 13.3801C8.90993 13.1342 8.97345 12.8974 9.25473 12.8792C9.54509 12.8519 9.63583 13.0705 9.67212 13.3164C9.69934 13.4985 9.73564 13.6716 9.74471 13.8538C9.74471 14.1088 9.73564 14.3638 9.73564 14.6279Z"></path>
                <path d="M0.00881775 12.0866C-0.0274771 10.1648 0.389914 8.4616 1.15211 6.83124C1.20655 6.70372 1.27007 6.58532 1.35173 6.47602C1.46061 6.33029 1.61487 6.25742 1.78727 6.3485C1.97782 6.43959 2.05041 6.60353 1.97782 6.80391C1.88708 7.05894 1.76912 7.30486 1.66024 7.55989C0.743789 9.72763 0.516946 11.9682 0.997854 14.2817C1.06137 14.5823 1.28821 15.0013 0.798232 15.1197C0.308251 15.2381 0.308251 14.7644 0.244735 14.4639C0.0814075 13.6441 -0.0365509 12.8153 0.00881775 12.0866Z"></path>
                <path d="M11.9136 4.29004C13.6648 4.3538 15.171 4.80921 16.5049 5.78378C16.5502 5.82021 16.6047 5.85665 16.65 5.89308C16.8497 6.07524 17.1309 6.25741 16.886 6.56708C16.65 6.87676 16.4051 6.66727 16.1782 6.50333C14.0731 4.95494 11.7956 4.69991 9.36385 5.61073C9.3094 5.62894 9.25496 5.65627 9.19145 5.67448C8.97368 5.75646 8.75591 5.75646 8.64702 5.51965C8.51999 5.25551 8.67424 5.08245 8.90109 4.97316C9.58162 4.65437 10.2894 4.47221 11.0243 4.37202C11.3601 4.33559 11.6867 4.31737 11.9136 4.29004Z"></path>
                <path d="M15.4521 16.5042C15.2797 17.9068 14.9531 19.2457 14.454 20.5391C14.0457 21.5865 13.5467 22.5884 12.9206 23.5266C12.8571 23.6268 12.7935 23.7361 12.7119 23.8271C12.5486 24.0184 12.3399 24.0548 12.1493 23.9273C11.9406 23.7907 11.9769 23.5903 12.0858 23.3991C12.2763 23.0712 12.4941 22.7615 12.6847 22.4336C13.71 20.6484 14.3724 18.7448 14.6355 16.6954C14.6446 16.5861 14.6718 16.486 14.699 16.3858C14.7535 16.1672 14.8896 16.0306 15.1255 16.0579C15.3705 16.0943 15.4521 16.2856 15.4521 16.5042Z"></path>
                <path d="M21.3501 14.7098C21.3773 16.4676 21.1414 18.1891 20.7331 19.8923C20.7149 19.947 20.7059 20.0107 20.6877 20.0654C20.6061 20.3295 20.5425 20.6574 20.1705 20.5572C19.8166 20.4661 19.8529 20.1655 19.9255 19.8832C20.1161 19.1454 20.2703 18.4077 20.3611 17.6517C20.5425 16.2673 20.6786 14.8828 20.5425 13.4802C20.5153 13.1978 20.5516 12.9246 20.8783 12.879C21.2503 12.8335 21.3229 13.134 21.332 13.4255C21.3592 13.8536 21.3501 14.2817 21.3501 14.7098Z"></path>
                <path d="M3.94694 14.5094C3.8925 16.0031 3.3753 17.1872 2.34089 18.1435C2.16849 18.2984 1.97794 18.3894 1.77832 18.2073C1.56963 18.016 1.59685 17.7792 1.7874 17.6061C3.15753 16.3583 3.34808 14.8099 2.96698 13.0976C2.91254 12.8517 2.92161 12.5875 2.92161 12.3325C2.93069 12.1321 3.04864 11.9773 3.25734 11.9773C3.46604 11.9682 3.61122 12.0957 3.63844 12.2961C3.7564 13.0612 3.85621 13.8263 3.94694 14.5094Z"></path>
                <path d="M10.3619 18.9908C10.2711 19.2002 10.1532 19.4735 10.0262 19.7467C9.48173 20.9035 8.78305 21.96 7.89382 22.8799C7.70328 23.0712 7.49458 23.2534 7.24052 23.0075C7.0046 22.7798 7.14978 22.5612 7.32218 22.379C8.30214 21.3316 9.05526 20.1384 9.60876 18.8086C9.6995 18.59 9.8356 18.3987 10.1169 18.4807C10.3256 18.5445 10.3891 18.7084 10.3619 18.9908Z"></path>
                <path d="M6.33312 17.2874C6.3059 17.3785 6.26054 17.506 6.21517 17.6244C5.75241 18.7174 5.05373 19.6282 4.17358 20.4115C3.96488 20.6028 3.71989 20.7121 3.50212 20.448C3.29343 20.202 3.46583 19.9835 3.65638 19.8286C4.50023 19.1091 5.11725 18.2256 5.54371 17.2055C5.64352 16.9778 5.77055 16.8047 6.05184 16.8594C6.23331 16.914 6.32405 17.0506 6.33312 17.2874Z"></path>
                <path d="M17.8026 19.5554C17.7753 19.6647 17.7572 19.7831 17.7209 19.9015C17.4215 20.9216 17.0676 21.9144 16.5957 22.8708C16.4778 23.1167 16.3145 23.3444 16.006 23.1987C15.7156 23.062 15.7882 22.7979 15.8971 22.5702C16.3508 21.5774 16.7409 20.5573 17.0313 19.5099C17.0948 19.2639 17.2218 19.0909 17.5122 19.1455C17.7209 19.1911 17.8026 19.3459 17.8026 19.5554Z"></path>
                </g>
              </g>
            </svg>
          </div>
        </div>

        <div class="auth-action-region mt-6">
          <div id="auth-passkey-view" class="auth-pane" aria-hidden="false">
            <div class="action-group" data-action-group="pair">
              <button
                id="passkey-primary"
                class="btn-primary h-12 w-full"
                data-action-role="primary"
              >
                Authenticate with passkey
              </button>
              <button
                id="toggle-email"
                class="btn-ghost h-12 w-full"
                data-action-role="secondary"
              >
                Sign in with email
              </button>
            </div>
          </div>

          <div id="auth-email-view" class="auth-pane hidden" aria-hidden="true">
            <form id="email-signin-form" class="space-y-3">
              <label class="field-label" for="signin-email">Email</label>
              <input id="signin-email" type="email" autocomplete="email" class="field-input h-12 w-full" />
              <label class="field-label" for="signin-password">Password</label>
              <input id="signin-password" type="password" autocomplete="current-password" class="field-input h-12 w-full" />
              <div class="action-group" data-action-group="pair">
                <button
                  id="back-passkey"
                  type="button"
                  class="btn-ghost h-12 w-full"
                  data-action-role="secondary"
                >
                  Back to passkey
                </button>
                <button
                  id="email-continue"
                  type="submit"
                  class="btn-primary h-12 w-full"
                  data-action-role="primary"
                >
                  Continue
                </button>
              </div>
            </form>
          </div>
        </div>

        <div id="auth-error" class="mt-3 hidden error-card"></div>
      </section>

      <button
        type="button"
        class="theme-mini-switch"
        data-theme-toggle
        data-theme-toggle-compact
        aria-label="Theme toggle"
      >
        <svg class="theme-mini-icon theme-mini-icon-sun" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
          <circle cx="12" cy="12" r="4"></circle>
          <line x1="12" y1="2" x2="12" y2="4.5"></line>
          <line x1="12" y1="19.5" x2="12" y2="22"></line>
          <line x1="4.9" y1="4.9" x2="6.7" y2="6.7"></line>
          <line x1="17.3" y1="17.3" x2="19.1" y2="19.1"></line>
          <line x1="2" y1="12" x2="4.5" y2="12"></line>
          <line x1="19.5" y1="12" x2="22" y2="12"></line>
          <line x1="4.9" y1="19.1" x2="6.7" y2="17.3"></line>
          <line x1="17.3" y1="6.7" x2="19.1" y2="4.9"></line>
        </svg>
        <svg class="theme-mini-icon theme-mini-icon-moon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
          <path d="M21 12.8A8.5 8.5 0 1 1 11.2 3a6.8 6.8 0 0 0 9.8 9.8z"></path>
        </svg>
        <span class="theme-mini-thumb" aria-hidden="true"></span>
      </button>
    </div>
  </div>
</main>

<script>
(() => {
  const passkeyBtn = document.getElementById('passkey-primary');
  const toggleEmailBtn = document.getElementById('toggle-email');
  const backPasskeyBtn = document.getElementById('back-passkey');
  const passkeyView = document.getElementById('auth-passkey-view');
  const emailView = document.getElementById('auth-email-view');
  const emailForm = document.getElementById('email-signin-form');
  const authError = document.getElementById('auth-error');
  const reduceMotionQuery = window.matchMedia('(prefers-reduced-motion: reduce)');
  const fadeDurationMs = reduceMotionQuery.matches ? 0 : 120;
  const PASSKEY_PROMPT_TIMEOUT_MS = 12000;
  const passkeyDefaultLabel = passkeyBtn ? passkeyBtn.textContent : 'Authenticate with passkey';
  let swapInProgress = false;

  const showError = (message) => {
    authError.textContent = message;
    authError.classList.remove('hidden');
  };

  const clearError = () => {
    authError.textContent = '';
    authError.classList.add('hidden');
  };

  const switchView = (target) => {
    const showingEmail = target === 'email';
    const fromView = showingEmail ? passkeyView : emailView;
    const toView = showingEmail ? emailView : passkeyView;

    if (!fromView || !toView || swapInProgress || fromView.classList.contains('hidden')) {
      return;
    }
    swapInProgress = true;
    fromView.classList.add('auth-pane-fade-out');
    const commitSwitch = () => {
      fromView.classList.add('hidden');
      fromView.setAttribute('aria-hidden', 'true');
      fromView.classList.remove('auth-pane-fade-out');

      toView.classList.remove('hidden');
      toView.setAttribute('aria-hidden', 'false');
      toView.classList.add('auth-pane-fade-in');
      requestAnimationFrame(() => {
        toView.classList.remove('auth-pane-fade-in');
      });
      swapInProgress = false;
    };

    if (fadeDurationMs === 0) {
      commitSwitch();
    } else {
      window.setTimeout(commitSwitch, fadeDurationMs);
    }
  };

  const showPasskeyView = () => {
    clearError();
    if (!passkeyView.classList.contains('hidden')) return;
    switchView('passkey');
  };

  const showEmailView = () => {
    clearError();
    if (!emailView.classList.contains('hidden')) return;
    switchView('email');
  };

  const requestJson = async (url, method, payload) => {
    const response = await fetch(url, {
      method,
      headers: { 'Content-Type': 'application/json', 'Accept': 'application/json' },
      body: payload ? JSON.stringify(payload) : undefined,
    });
    const bodyText = await response.text();
    let body = null;
    try { body = bodyText ? JSON.parse(bodyText) : null; } catch (_err) {}
    return { ok: response.ok, status: response.status, body, bodyText };
  };

  const b64urlToBuffer = (value) => {
    const padded = (value + '==='.slice((value.length + 3) % 4)).replace(/-/g, '+').replace(/_/g, '/');
    const binary = atob(padded);
    const bytes = new Uint8Array(binary.length);
    for (let i = 0; i < binary.length; i += 1) bytes[i] = binary.charCodeAt(i);
    return bytes.buffer;
  };

  const bufferToB64url = (buffer) => {
    const bytes = new Uint8Array(buffer);
    let binary = '';
    for (let i = 0; i < bytes.length; i += 1) binary += String.fromCharCode(bytes[i]);
    return btoa(binary).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
  };

  const setPasskeyBusy = (busy, label) => {
    if (!passkeyBtn) return;
    passkeyBtn.disabled = busy;
    passkeyBtn.setAttribute('aria-busy', busy ? 'true' : 'false');
    passkeyBtn.textContent = busy ? label : passkeyDefaultLabel;
  };

  const serializeAuthCredential = (credential) => ({
    id: credential.id,
    rawId: bufferToB64url(credential.rawId),
    type: credential.type,
    response: {
      authenticatorData: bufferToB64url(credential.response.authenticatorData),
      clientDataJSON: bufferToB64url(credential.response.clientDataJSON),
      signature: bufferToB64url(credential.response.signature),
      userHandle: credential.response.userHandle ? bufferToB64url(credential.response.userHandle) : null,
    },
    clientExtensionResults: credential.getClientExtensionResults ? credential.getClientExtensionResults() : {},
  });

  const getPasskeyCredential = async (options) => {
    if (typeof AbortController !== 'function') {
      return navigator.credentials.get(options);
    }
    const controller = new AbortController();
    const timeoutId = window.setTimeout(() => controller.abort(), PASSKEY_PROMPT_TIMEOUT_MS);
    try {
      return await navigator.credentials.get({ ...options, signal: controller.signal });
    } finally {
      window.clearTimeout(timeoutId);
    }
  };

  const passkeyAuth = async () => {
    if (!passkeyBtn || passkeyBtn.disabled) return;
    clearError();
    setPasskeyBusy(true, 'Preparing passkey...');
    try {
      const start = await requestJson('/api/auth/passkeys/auth/start', 'POST', {});
      if (!start.ok || !start.body || !start.body.options || !start.body.flow_id) {
        const requestId = start.body && start.body.request_id ? ` (request_id: ${start.body.request_id})` : '';
        showError(`Passkey start failed${requestId}`);
        return;
      }
      if (!window.PublicKeyCredential || !navigator.credentials) {
        showError('WebAuthn is not supported in this browser.');
        return;
      }
      const options = typeof structuredClone === 'function'
        ? structuredClone(start.body.options)
        : JSON.parse(JSON.stringify(start.body.options));
      if (options.mediation === 'conditional') {
        options.mediation = 'required';
      }
      options.publicKey.challenge = b64urlToBuffer(options.publicKey.challenge);
      if (Array.isArray(options.publicKey.allowCredentials)) {
        options.publicKey.allowCredentials = options.publicKey.allowCredentials.map((item) => ({ ...item, id: b64urlToBuffer(item.id) }));
      }
      setPasskeyBusy(true, 'Waiting for passkey...');
      const credential = await getPasskeyCredential(options);
      if (!credential) {
        showError('Passkey authentication was cancelled.');
        return;
      }
      const finish = await requestJson('/api/auth/passkeys/auth/finish', 'POST', {
        flow_id: start.body.flow_id,
        credential: serializeAuthCredential(credential),
      });
      if (!finish.ok) {
        const requestId = finish.body && finish.body.request_id ? ` (request_id: ${finish.body.request_id})` : '';
        showError(`Passkey sign-in failed${requestId}`);
        return;
      }
      window.location.href = '/dashboard';
    } catch (err) {
      if (err && typeof err === 'object' && err.name === 'SecurityError') {
        showEmailView();
        showError('Passkey is unavailable on this host. Use email/password sign-in.');
        return;
      }
      if (err && typeof err === 'object' && err.name === 'NotAllowedError') {
        showEmailView();
        showError('Passkey sign-in was cancelled. Continue with email sign-in.');
        return;
      }
      if (err && typeof err === 'object' && err.name === 'AbortError') {
        showEmailView();
        showError('Passkey sign-in timed out. Continue with email sign-in.');
        return;
      }
      showError('Passkey authentication failed. Use email/password if needed.');
    } finally {
      setPasskeyBusy(false);
    }
  };

  passkeyBtn.addEventListener('click', passkeyAuth);
  toggleEmailBtn.addEventListener('click', showEmailView);
  backPasskeyBtn.addEventListener('click', showPasskeyView);

  emailForm.addEventListener('submit', async (event) => {
    event.preventDefault();
    clearError();
    const email = document.getElementById('signin-email').value.trim();
    const password = document.getElementById('signin-password').value;
    const result = await requestJson('/api/auth/password/signin', 'POST', { email, password });
    if (!result.ok) {
      const requestId = result.body && result.body.request_id ? ` (request_id: ${result.body.request_id})` : '';
      showError(`Sign-in failed${requestId}`);
      return;
    }
    window.location.href = '/dashboard';
  });

})();
</script>
"#;
    html.to_string()
}

pub fn render_dashboard_overview(email: &str) -> String {
    let topbar = app_shell_topbar("Dashboard", &format!("Signed in as {}", html_escape(email)));
    let sidebar = dashboard_desktop_sidebar("home");
    format!(
        r#"{topbar}
<main class="mx-auto w-full max-w-[480px] px-4 pb-24 pt-4 md:max-w-7xl md:px-6 md:pt-8">
  <div class="md:grid md:grid-cols-[220px_minmax(0,1fr)] md:items-start md:gap-4">
    {sidebar}
    <section class="glass-card rounded-[22px] p-5">
      <h2 class="text-lg font-semibold txt-strong">Operational summary</h2>
      <div id="dashboard-summary" class="mt-4 space-y-2">
        <div class="summary-row"><span>Total jobs</span><span>--</span></div>
        <div class="summary-row"><span>Queued</span><span>--</span></div>
        <div class="summary-row"><span>Running</span><span>--</span></div>
        <div class="summary-row"><span>Failed</span><span>--</span></div>
      </div>
      <div class="action-group" data-action-group="single">
        <a
          href="/dashboard/jobs"
          class="btn-primary inline-flex h-12 w-full items-center justify-center"
          data-action-role="primary"
        >
          View Jobs
        </a>
      </div>
    </section>
  </div>
</main>

<nav class="bottom-nav">
  <a href="/dashboard" class="active">Home</a>
  <a href="/dashboard/jobs">Jobs</a>
  <a href="/dashboard/security">Security</a>
</nav>

<script>
(() => {{
  const summary = document.getElementById('dashboard-summary');
  fetch('/api/dashboard/summary', {{ headers: {{ Accept: 'application/json' }} }})
    .then((res) => res.json().then((json) => [res.ok, json]))
    .then(([ok, data]) => {{
      if (!ok) {{
        summary.innerHTML = `<div class="error-card">Failed to load summary. request_id: ${{data.request_id || 'n/a'}}</div>`;
        return;
      }}
      summary.innerHTML = `
        <div class="summary-row"><span>Total jobs</span><span>${{data.total_jobs}}</span></div>
        <div class="summary-row"><span>Queued</span><span>${{data.queued_jobs}}</span></div>
        <div class="summary-row"><span>Running</span><span>${{data.running_jobs}}</span></div>
        <div class="summary-row"><span>Failed</span><span>${{data.failed_jobs}}</span></div>
        <div class="support-row">Support code: ${{data.support_request_id}}</div>`;
    }})
    .catch((err) => {{
      summary.innerHTML = `<div class="error-card">${{String(err)}}</div>`;
    }});
}})();
</script>"#
    )
}

pub fn render_dashboard_jobs(email: &str) -> String {
    let topbar = app_shell_topbar("Jobs", &format!("Signed in as {}", html_escape(email)));
    let sidebar = dashboard_desktop_sidebar("jobs");
    format!(
        r#"{topbar}
<main class="mx-auto w-full max-w-[480px] px-4 pb-24 pt-4 md:max-w-7xl md:px-6 md:pt-8">
  <div class="md:grid md:grid-cols-[220px_minmax(0,1fr)] md:items-start md:gap-4">
    {sidebar}
    <section class="glass-card rounded-[22px] p-5">
      <h2 class="text-lg font-semibold txt-strong">My runtime jobs</h2>
      <div id="jobs-list" class="mt-4 space-y-2"><div class="loading-card">Loading jobs...</div></div>
    </section>
  </div>
</main>
<nav class="bottom-nav">
  <a href="/dashboard">Home</a>
  <a href="/dashboard/jobs" class="active">Jobs</a>
  <a href="/dashboard/security">Security</a>
</nav>
<script>
(() => {{
  const list = document.getElementById('jobs-list');
  const renderJobs = (jobs) => {{
    if (!jobs.length) {{
      list.innerHTML = '<div class="empty-card">No jobs available.</div>';
      return;
    }}
    list.innerHTML = jobs.map((job) => `
      <a class="summary-card" href="/dashboard/jobs/${{job.job_id}}">
        <div class="summary-row"><span>Job</span><span>${{job.job_id}}</span></div>
        <div class="summary-row"><span>Status</span><span class="badge">${{job.status}}</span></div>
      </a>
    `).join('');
  }};
  fetch('/api/dashboard/jobs', {{ headers: {{ Accept: 'application/json' }} }})
    .then((res) => res.json().then((json) => [res.ok, json]))
    .then(([ok, data]) => {{
      if (!ok) {{
        list.innerHTML = `<div class="error-card">Failed to load jobs. request_id: ${{data.request_id || 'n/a'}}</div>`;
        return;
      }}
      renderJobs(data.jobs || []);
    }})
    .catch((err) => {{
      list.innerHTML = `<div class="error-card">${{String(err)}}</div>`;
    }});
}})();
</script>"#
    )
}

pub fn render_dashboard_job_detail(email: &str, job_id: &str) -> String {
    let topbar = app_shell_topbar(
        "Job detail",
        &format!("{} · {}", html_escape(email), html_escape(job_id)),
    );
    let sidebar = dashboard_desktop_sidebar("jobs");
    let escaped_job_id = html_escape(job_id);
    format!(
        r#"{topbar}
<main class="mx-auto w-full max-w-[480px] px-4 pb-28 pt-4 md:max-w-7xl md:px-6 md:pt-8">
  <div class="md:grid md:grid-cols-[220px_minmax(0,1fr)] md:items-start md:gap-4">
    {sidebar}
    <section class="glass-card rounded-[22px] p-5">
      <div class="summary-row"><span>Job ID</span><span id="job-id">{escaped_job_id}</span></div>
      <div class="summary-row"><span>Status</span><span id="job-status">--</span></div>
      <div id="events" class="mt-4 space-y-2"></div>
    </section>
  </div>
</main>
<div class="action-sticky" data-action-group="sticky">
  <a
    href="/dashboard/jobs"
    class="btn-ghost h-12 w-full text-center leading-[3rem]"
    data-action-role="secondary"
  >
    Back
  </a>
  <button
    id="manual-refresh"
    class="btn-primary h-12 w-full"
    data-action-role="primary"
  >
    Refresh
  </button>
</div>
<script>
(() => {{
  const jobId = document.getElementById('job-id').textContent.trim();
  const statusEl = document.getElementById('job-status');
  const eventsEl = document.getElementById('events');
  const refreshBtn = document.getElementById('manual-refresh');
  let lastEventId = 0;
  let fallbackInterval = null;

  const renderEvents = (events) => {{
    if (!events.length && !eventsEl.innerHTML.trim()) {{
      eventsEl.innerHTML = '<div class="empty-card">No events yet.</div>';
      return;
    }}
    events.forEach((event) => {{
      lastEventId = Math.max(lastEventId, Number(event.id || 0));
      const node = document.createElement('div');
      node.className = 'summary-card';
      node.innerHTML = `<div class="summary-row"><span>${{event.event_type}}</span><span>${{event.id}}</span></div>`;
      eventsEl.prepend(node);
    }});
  }};

  const loadJob = () => fetch(`/api/dashboard/jobs/${{jobId}}`, {{ headers: {{ Accept: 'application/json' }} }})
    .then((res) => res.json().then((json) => [res.ok, json]))
    .then(([ok, data]) => {{
      if (!ok) {{
        statusEl.textContent = `error (request_id: ${{data.request_id || 'n/a'}})`;
        return;
      }}
      statusEl.textContent = data.status;
    }});

  const pollEvents = () => fetch(`/api/dashboard/jobs/${{jobId}}/events?since_id=${{lastEventId}}`, {{ headers: {{ Accept: 'application/json' }} }})
    .then((res) => res.json())
    .then((data) => renderEvents(data.events || []))
    .catch(() => {{}});

  const startFallback = () => {{
    if (fallbackInterval) return;
    fallbackInterval = setInterval(pollEvents, 10000);
  }};

  const startSse = () => {{
    if (!window.EventSource) {{
      startFallback();
      return;
    }}
    const source = new EventSource(`/api/dashboard/jobs/${{jobId}}/events/stream?since_id=${{lastEventId}}`);
    source.addEventListener('job_event', (event) => {{
      try {{
        const payload = JSON.parse(event.data);
        renderEvents([payload]);
      }} catch (_err) {{}}
    }});
    source.onerror = () => {{
      source.close();
      startFallback();
    }};
  }};

  refreshBtn.addEventListener('click', () => {{
    loadJob();
    pollEvents();
  }});

  loadJob();
  pollEvents();
  startSse();
}})();
</script>"#
    )
}

pub fn render_dashboard_security(email: &str) -> String {
    let topbar = app_shell_topbar("Security", &format!("Signed in as {}", html_escape(email)));
    let sidebar = dashboard_desktop_sidebar("security");
    format!(
        r#"{topbar}
<main class="mx-auto w-full max-w-[480px] px-4 pb-24 pt-4 md:max-w-7xl md:px-6 md:pt-8">
  <div class="md:grid md:grid-cols-[220px_minmax(0,1fr)] md:items-start md:gap-4">
    {sidebar}
    <section class="glass-card rounded-[22px] p-5">
      <h2 class="text-lg font-semibold txt-strong">Account settings</h2>

      <div class="mt-4 space-y-4">
        <section class="summary-card p-4">
          <div class="flex items-center justify-between gap-2">
            <h3 class="text-sm font-semibold txt-strong">Passkeys</h3>
            <button
              id="passkey-manage"
              type="button"
              class="btn-chip inline-flex h-9 w-9 items-center justify-center rounded-xl p-0"
              aria-label="Edit selected passkey"
              title="Edit selected passkey"
              disabled
            >
              <svg class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                <path d="M12 20h9"></path>
                <path d="M16.5 3.5a2.1 2.1 0 0 1 3 3L8 18l-4 1 1-4 11.5-11.5z"></path>
              </svg>
            </button>
          </div>
          <div id="passkey-status" class="mt-3 hidden"></div>
          <div id="passkeys-list" class="mt-4 space-y-2"><div class="loading-card">Loading passkeys...</div></div>
          <div class="action-group" data-action-group="single">
            <button
              id="create-passkey"
              type="button"
              class="btn-primary h-11 w-full px-4"
              data-action-role="primary"
            >
              Create passkey
            </button>
          </div>
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
          <h4 id="passkey-modal-title" class="text-base font-semibold txt-strong">Edit passkey</h4>
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
          <div class="action-destructive" data-action-group="destructive">
            <button
              id="passkey-modal-delete"
              type="button"
              class="btn-destructive h-11 w-full"
              data-action-role="destructive"
            >
              Delete passkey
            </button>
          </div>
        </div>
      </div>
    </section>
  </div>
</main>
<nav class="bottom-nav">
  <a href="/dashboard">Home</a>
  <a href="/dashboard/jobs">Jobs</a>
  <a href="/dashboard/security" class="active">Security</a>
</nav>
<script>
(() => {{
  const list = document.getElementById('passkeys-list');
  const passkeyStatus = document.getElementById('passkey-status');
  const passkeyManageButton = document.getElementById('passkey-manage');
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
  const createPasskeyButton = document.getElementById('create-passkey');
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

  const closeSecurityModal = (result) => {{
    if (!modalResolver || !securityModal) {{
      return;
    }}
    const resolve = modalResolver;
    modalResolver = null;
    securityModal.classList.add('hidden');
    securityModal.setAttribute('aria-hidden', 'true');
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
    securityModal.classList.remove('hidden');
    securityModal.setAttribute('aria-hidden', 'false');
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
    passkeyModal.classList.add('hidden');
    passkeyModal.setAttribute('aria-hidden', 'true');
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
    passkeyModal.classList.remove('hidden');
    passkeyModal.setAttribute('aria-hidden', 'false');
    requestAnimationFrame(() => {{
      passkeyModalLabelInput.focus();
      passkeyModalLabelInput.select();
    }});
  }};

  const renderSelectedPasskey = () => {{
    const selected = selectedCredentialId ? passkeysById.get(selectedCredentialId) : null;
    if (passkeyManageButton) {{
      passkeyManageButton.disabled = !selected;
    }}
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
    renderSelectedPasskey();
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
      if (!selectedCredentialId) {{
        selectedCredentialId = passkeys[0].credential_id;
      }}
      list.innerHTML = passkeys.map((item) => `
        <button type="button" class="summary-card passkey-card w-full text-left" data-credential-id="${{item.credential_id}}" aria-pressed="false">
          <div class="summary-row">
            <span>${{escapeHtml(effectivePasskeyLabel(item))}}</span>
            <span class="txt-supporting">${{item.last_used_at ? `Used ${{formatTimestamp(item.last_used_at)}}` : 'Never used'}}</span>
          </div>
        </button>
      `).join('');
      list.querySelectorAll('[data-credential-id]').forEach((card) => {{
        card.addEventListener('click', () => {{
          selectedCredentialId = card.dataset.credentialId || null;
          syncPasskeySelection();
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
    passkeyLabelOverrides.set(selectedCredentialId, friendlyName);
    persistPasskeyLabelOverrides();
    showStatus(passkeyStatus, 'Passkey label updated for this browser.', 'success');
    syncPasskeySelection();
    await load();
    if (passkeyModalSaveButton) {{
      passkeyModalSaveButton.disabled = false;
    }}
  }};

  const createPasskey = async () => {{
    clearStatus(passkeyStatus);
    if (!window.PublicKeyCredential || !navigator.credentials) {{
      showStatus(passkeyStatus, 'Passkeys are unavailable in this browser.', 'error');
      return;
    }}

    createPasskeyButton.disabled = true;
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
      createPasskeyButton.disabled = false;
    }}
  }};

  createPasskeyButton.addEventListener('click', (event) => {{
    event.preventDefault();
    createPasskey();
  }});
  if (passkeyManageButton) {{
    passkeyManageButton.addEventListener('click', openPasskeyModal);
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
    if (passkeyModal && !passkeyModal.classList.contains('hidden') && event.key === 'Escape') {{
      event.preventDefault();
      closePasskeyModal();
      return;
    }}
    if (!securityModal || securityModal.classList.contains('hidden')) {{
      return;
    }}
    if (event.key === 'Escape') {{
      event.preventDefault();
      closeSecurityModal({{ confirmed: false, value: '' }});
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
  load();
}})();
</script>"#
    )
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
          <a class="btn-ghost h-12 w-full text-center leading-[3rem]" href="{}">/health</a>
          <a class="btn-ghost h-12 w-full text-center leading-[3rem]" href="{}">/ready</a>
        </div>
        <div class="action-group action-pair" data-action-group="pair">
          <a class="btn-ghost h-12 w-full text-center leading-[3rem]" href="{}">metrics text</a>
          <a class="btn-primary h-12 w-full text-center leading-[3rem]" href="/admin/observability">Open observability</a>
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
    format!(
        r#"<aside class="hidden md:sticky md:top-6 md:block md:self-start">
  <div class="glass-card rounded-[22px] p-3">
    <p class="eyebrow px-3 pt-1">ops navigation</p>
    <nav class="mt-2 space-y-1">
      <a href="/admin/maintenance" class="desktop-side-link {}">Maintenance</a>
      <a href="/admin/users" class="desktop-side-link {}">Users</a>
      <a href="/admin/runtime" class="desktop-side-link {}">Runtime</a>
      <a href="/admin/observability" class="desktop-side-link {}">Observability</a>
      <a href="/admin/security" class="desktop-side-link {}">Security</a>
      <a href="/admin/config" class="desktop-side-link {}">Config</a>
      <a href="/admin/audit" class="desktop-side-link {}">Audit</a>
    </nav>
  </div>
</aside>"#,
        if active == "maintenance" {
            "active"
        } else {
            ""
        },
        if active == "users" { "active" } else { "" },
        if active == "runtime" { "active" } else { "" },
        if active == "observability" {
            "active"
        } else {
            ""
        },
        if active == "security" { "active" } else { "" },
        if active == "config" { "active" } else { "" },
        if active == "audit" { "active" } else { "" },
    )
}

fn admin_bottom_nav(active: &str) -> String {
    format!(
        r#"<nav class="bottom-nav">
  <a href="/admin/maintenance" class="{}">Maint</a>
  <a href="/admin/users" class="{}">Users</a>
  <a href="/admin/runtime" class="{}">Runtime</a>
  <a href="/admin/observability" class="{}">Obs</a>
  <a href="/admin/audit" class="{}">Audit</a>
</nav>"#,
        if active == "maintenance" {
            "active"
        } else {
            ""
        },
        if active == "users" { "active" } else { "" },
        if active == "runtime" { "active" } else { "" },
        if active == "observability" {
            "active"
        } else {
            ""
        },
        if active == "audit" { "active" } else { "" },
    )
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

const ADMIN_SECTION_SCRIPT: &str = r##"
<script>
(() => {
  const shell = document.querySelector('[data-admin-section]');
  if (!shell) return;
  const section = shell.getAttribute('data-admin-section') || '';
  const content = document.getElementById('admin-content');
  const flash = document.getElementById('admin-flash');
  const modal = document.getElementById('admin-confirm-modal');
  const modalTitle = document.getElementById('admin-confirm-title');
  const modalMessage = document.getElementById('admin-confirm-message');
  const modalTarget = document.getElementById('admin-confirm-target');
  const modalReason = document.getElementById('admin-confirm-reason');
  const modalCancel = document.getElementById('admin-confirm-cancel');
  const modalSubmit = document.getElementById('admin-confirm-submit');
  const chartScriptUrl = '/assets/lightweight-charts.standalone.production.js';

  if (!content || !flash || !modal || !modalTitle || !modalMessage || !modalTarget || !modalReason || !modalCancel || !modalSubmit) {
    return;
  }

  const escapeHtml = (value) => String(value ?? '')
    .replaceAll('&', '&amp;')
    .replaceAll('<', '&lt;')
    .replaceAll('>', '&gt;')
    .replaceAll('"', '&quot;')
    .replaceAll("'", '&#39;');

  const asText = (value, fallback = 'n/a') => {
    if (value === null || value === undefined || value === '') return fallback;
    return String(value);
  };

  const formatDate = (value) => {
    if (!value) return 'n/a';
    const parsed = new Date(value);
    if (Number.isNaN(parsed.getTime())) return 'n/a';
    return parsed.toLocaleString();
  };

  const setFlash = (kind, message) => {
    if (!message) {
      flash.textContent = '';
      flash.className = 'mt-3 hidden';
      return;
    }
    flash.className = kind === 'error' ? 'mt-3 error-card' : 'mt-3 empty-card';
    flash.textContent = message;
  };

  const requestJson = async (url, method = 'GET', payload = null) => {
    const options = {
      method,
      headers: {
        Accept: 'application/json',
      },
    };
    if (payload !== null) {
      options.headers['Content-Type'] = 'application/json';
      options.body = JSON.stringify(payload);
    }
    let response;
    try {
      response = await fetch(url, options);
    } catch (error) {
      return { ok: false, status: 0, body: { message: String(error), request_id: 'n/a' } };
    }
    let body = null;
    try {
      body = await response.json();
    } catch (_error) {
      body = null;
    }
    return { ok: response.ok, status: response.status, body };
  };

  const errorMessage = (result) => {
    const body = result && result.body ? result.body : {};
    const message = body.message || 'Request failed';
    const requestId = body.request_id ? ` (request_id: ${body.request_id})` : '';
    return `${message}${requestId}`;
  };

  let modalResolver = null;
  let runtimeJobsStream = null;
  let runtimeStreamDisabled = false;

  const closeRuntimeJobsStream = () => {
    if (runtimeJobsStream) {
      runtimeJobsStream.close();
      runtimeJobsStream = null;
    }
  };
  const closeConfirmModal = (value) => {
    modal.classList.add('hidden');
    if (modalResolver) {
      modalResolver(value);
      modalResolver = null;
    }
  };

  const openConfirmModal = ({ title, message, targetLabel, confirmText }) => new Promise((resolve) => {
    modalResolver = resolve;
    modalTitle.textContent = title || 'Confirm action';
    modalMessage.textContent = message || '';
    modalTarget.value = '';
    modalTarget.placeholder = targetLabel || '';
    modalReason.value = '';
    modalSubmit.textContent = confirmText || 'Confirm';
    modal.classList.remove('hidden');
    window.setTimeout(() => modalTarget.focus(), 0);
  });

  modalCancel.addEventListener('click', () => closeConfirmModal(null));
  modalSubmit.addEventListener('click', () => {
    const payload = {
      confirm_target: modalTarget.value.trim(),
      reason: modalReason.value.trim(),
    };
    if (!payload.confirm_target || !payload.reason) {
      return;
    }
    closeConfirmModal(payload);
  });
  modal.addEventListener('click', (event) => {
    if (event.target === modal) closeConfirmModal(null);
  });
  document.addEventListener('keydown', (event) => {
    if (event.key === 'Escape' && !modal.classList.contains('hidden')) {
      closeConfirmModal(null);
    }
  });

  const renderUsers = async () => {
    const [usersResult, sessionsResult] = await Promise.all([
      requestJson('/api/admin/users'),
      requestJson('/api/admin/sessions'),
    ]);
    if (!usersResult.ok) {
      content.innerHTML = `<div class="error-card">${escapeHtml(errorMessage(usersResult))}</div>`;
      return;
    }
    if (!sessionsResult.ok) {
      content.innerHTML = `<div class="error-card">${escapeHtml(errorMessage(sessionsResult))}</div>`;
      return;
    }
    const users = Array.isArray(usersResult.body?.users) ? usersResult.body.users : [];
    const sessions = Array.isArray(sessionsResult.body?.sessions) ? sessionsResult.body.sessions : [];
    const userRows = users.map((user) => {
      const accessLabel = user.access_enabled ? 'Disable access' : 'Enable access';
      const nextAccess = user.access_enabled ? 'false' : 'true';
      return `
        <article class="admin-row" data-user-id="${escapeHtml(user.user_id)}">
          <div class="min-w-0">
            <p class="truncate text-sm font-semibold txt-strong">${escapeHtml(user.email)}</p>
            <p class="truncate text-xs txt-faint">${escapeHtml(user.user_id)}</p>
            <p class="mt-1 text-xs txt-supporting">Status: ${escapeHtml(asText(user.status))} · Role: ${escapeHtml(asText(user.role))}</p>
          </div>
          <div class="admin-row-actions">
            <select class="field-input h-10 w-full md:w-[130px]" data-role-select>
              ${['user', 'viewer', 'operator', 'admin'].map((role) => `<option value="${role}" ${role === user.role ? 'selected' : ''}>${role}</option>`).join('')}
            </select>
            <button class="btn-ghost h-10 w-full md:w-auto" data-action="role">Update role</button>
            <button class="btn-ghost h-10 w-full md:w-auto" data-action="access" data-next-access="${nextAccess}">${accessLabel}</button>
            <button class="btn-destructive h-10 w-full md:w-auto" data-action="revoke">Revoke sessions</button>
          </div>
        </article>
      `;
    }).join('');
    const sessionRows = sessions.slice(0, 20).map((session) => `
      <div class="summary-row">
        <span class="truncate">${escapeHtml(asText(session.email))}</span>
        <span class="text-xs txt-supporting">${escapeHtml(asText(session.role))} · ${escapeHtml(formatDate(session.last_seen_at))}</span>
      </div>
    `).join('');
    content.innerHTML = `
      <section class="space-y-2">
        <h2 class="text-lg font-semibold txt-strong">User/role management</h2>
        ${userRows || '<div class="empty-card">No users found.</div>'}
      </section>
      <section class="space-y-2 pt-2">
        <h3 class="text-base font-semibold txt-strong">Recent sessions</h3>
        ${sessionRows || '<div class="empty-card">No sessions found.</div>'}
      </section>
    `;
    content.onclick = async (event) => {
      const button = event.target.closest('button[data-action]');
      if (!button) return;
      const row = button.closest('[data-user-id]');
      if (!row) return;
      const userId = row.getAttribute('data-user-id');
      if (!userId) return;
      const action = button.getAttribute('data-action');
      if (action === 'role') {
        const roleSelect = row.querySelector('[data-role-select]');
        const role = roleSelect ? roleSelect.value : 'user';
        const result = await openConfirmModal({
          title: 'Update role',
          message: `Type ${userId} and provide a reason to update role.`,
          targetLabel: userId,
          confirmText: 'Apply change',
        });
        if (!result) return;
        const response = await requestJson(`/api/admin/users/${encodeURIComponent(userId)}/role`, 'PATCH', {
          role,
          reason: result.reason,
          confirm_target: result.confirm_target,
        });
        if (!response.ok) {
          setFlash('error', errorMessage(response));
          return;
        }
        setFlash('success', 'User role updated.');
        await renderUsers();
        return;
      }
      if (action === 'access') {
        const accessEnabled = button.getAttribute('data-next-access') === 'true';
        const result = await openConfirmModal({
          title: 'Update user access',
          message: `Type ${userId} and provide a reason to update access.`,
          targetLabel: userId,
          confirmText: 'Apply change',
        });
        if (!result) return;
        const response = await requestJson(`/api/admin/users/${encodeURIComponent(userId)}/access`, 'PATCH', {
          access_enabled: accessEnabled,
          reason: result.reason,
          confirm_target: result.confirm_target,
        });
        if (!response.ok) {
          setFlash('error', errorMessage(response));
          return;
        }
        setFlash('success', 'User access updated.');
        await renderUsers();
        return;
      }
      if (action === 'revoke') {
        const result = await openConfirmModal({
          title: 'Revoke sessions',
          message: `Type ${userId} and provide a reason to revoke sessions.`,
          targetLabel: userId,
          confirmText: 'Revoke',
        });
        if (!result) return;
        const response = await requestJson(`/api/admin/users/${encodeURIComponent(userId)}/sessions/revoke`, 'POST', result);
        if (!response.ok) {
          setFlash('error', errorMessage(response));
          return;
        }
        setFlash('success', `Revoked ${asText(response.body?.revoked, '0')} sessions.`);
        await renderUsers();
      }
    };
  };

  const runtimeState = {
    jobs: [],
    flags: [],
  };

  const renderRuntimeMarkup = () => {
    const jobRows = runtimeState.jobs.slice(0, 120).map((job) => `
      <article class="admin-row" data-job-id="${escapeHtml(job.job_id)}">
        <div class="min-w-0">
          <p class="truncate text-sm font-semibold txt-strong">${escapeHtml(job.job_id)}</p>
          <p class="mt-1 text-xs txt-supporting">Status: ${escapeHtml(job.status)} · Attempts: ${escapeHtml(job.attempt_count)} · Updated: ${escapeHtml(formatDate(job.updated_at))}</p>
        </div>
        <div class="admin-row-actions">
          <button class="btn-ghost h-10 w-full md:w-auto" data-job-action="retry">Retry</button>
          <button class="btn-ghost h-10 w-full md:w-auto" data-job-action="requeue">Requeue</button>
          <button class="btn-destructive h-10 w-full md:w-auto" data-job-action="cancel">Cancel</button>
        </div>
      </article>
    `).join('');
    const flagRows = runtimeState.flags.map((flag) => `
      <article class="summary-row" data-flag="${escapeHtml(flag.flag)}">
        <span>${escapeHtml(flag.flag)}</span>
        <button class="btn-ghost h-10 px-3" data-flag-action="${flag.enabled ? 'disable' : 'enable'}">
          ${flag.enabled ? 'Disable' : 'Enable'}
        </button>
      </article>
    `).join('');
    content.innerHTML = `
      <section class="space-y-2">
        <h2 class="text-lg font-semibold txt-strong">Runtime jobs</h2>
        ${jobRows || '<div class="empty-card">No runtime jobs available.</div>'}
      </section>
      <section class="space-y-2 pt-2">
        <h3 class="text-base font-semibold txt-strong">Kill switches</h3>
        ${flagRows || '<div class="empty-card">No kill switches found.</div>'}
      </section>
    `;
  };

  const attachRuntimeHandlers = () => {
    content.onclick = async (event) => {
      const jobButton = event.target.closest('[data-job-action]');
      if (jobButton) {
        const row = jobButton.closest('[data-job-id]');
        const jobId = row ? row.getAttribute('data-job-id') : null;
        const action = jobButton.getAttribute('data-job-action');
        if (!jobId || !action) return;
        const payload = await openConfirmModal({
          title: `Runtime ${action}`,
          message: `Type ${jobId} and provide a reason for this runtime action.`,
          targetLabel: jobId,
          confirmText: 'Apply change',
        });
        if (!payload) return;
        const response = await requestJson(`/api/admin/runtime/jobs/${encodeURIComponent(jobId)}/${action}`, 'POST', payload);
        if (!response.ok) {
          setFlash('error', errorMessage(response));
          return;
        }
        setFlash('success', `Job ${jobId} updated.`);
        await renderRuntime();
        return;
      }
      const flagButton = event.target.closest('[data-flag-action]');
      if (!flagButton) return;
      const row = flagButton.closest('[data-flag]');
      const flag = row ? row.getAttribute('data-flag') : null;
      if (!flag) return;
      const enabled = flagButton.getAttribute('data-flag-action') === 'enable';
      const payload = await openConfirmModal({
        title: 'Update kill switch',
        message: `Type ${flag} and provide a reason to update the kill switch.`,
        targetLabel: flag,
        confirmText: 'Apply change',
      });
      if (!payload) return;
      const response = await requestJson(`/api/admin/runtime/kill-switches/${encodeURIComponent(flag)}`, 'PUT', {
        enabled,
        reason: payload.reason,
        confirm_target: payload.confirm_target,
      });
      if (!response.ok) {
        setFlash('error', errorMessage(response));
        return;
      }
      setFlash('success', `Kill switch ${flag} updated.`);
      await renderRuntime();
    };
  };

  const openRuntimeJobsStream = () => {
    if (runtimeStreamDisabled || !window.EventSource) {
      return;
    }
    closeRuntimeJobsStream();
    try {
      runtimeJobsStream = new EventSource('/api/admin/runtime/jobs/stream');
    } catch (_error) {
      runtimeStreamDisabled = true;
      return;
    }

    runtimeJobsStream.addEventListener('runtime_jobs', (event) => {
      let payload = null;
      try {
        payload = JSON.parse(event.data || '{}');
      } catch (_error) {
        return;
      }
      const jobs = Array.isArray(payload.jobs) ? payload.jobs : [];
      runtimeState.jobs = jobs;
      renderRuntimeMarkup();
      attachRuntimeHandlers();
    });

    runtimeJobsStream.addEventListener('error', () => {
      closeRuntimeJobsStream();
      // Keep the UI responsive if stream is unavailable; actions still refresh state.
      runtimeStreamDisabled = true;
    });
  };

  const renderRuntime = async () => {
    const [jobsResult, flagsResult] = await Promise.all([
      requestJson('/api/admin/runtime/jobs'),
      requestJson('/api/admin/runtime/kill-switches'),
    ]);
    if (!jobsResult.ok) {
      content.innerHTML = `<div class="error-card">${escapeHtml(errorMessage(jobsResult))}</div>`;
      return;
    }
    if (!flagsResult.ok) {
      content.innerHTML = `<div class="error-card">${escapeHtml(errorMessage(flagsResult))}</div>`;
      return;
    }
    runtimeState.jobs = Array.isArray(jobsResult.body?.jobs) ? jobsResult.body.jobs : [];
    runtimeState.flags = Array.isArray(flagsResult.body?.flags) ? flagsResult.body.flags : [];
    renderRuntimeMarkup();
    attachRuntimeHandlers();
    openRuntimeJobsStream();
  };

  const ensureLightweightCharts = async () => {
    if (window.LightweightCharts) return window.LightweightCharts;
    if (!window.__bominalLwPromise) {
      window.__bominalLwPromise = new Promise((resolve, reject) => {
        const script = document.createElement('script');
        script.src = chartScriptUrl;
        script.async = true;
        script.onload = () => resolve(window.LightweightCharts);
        script.onerror = () => reject(new Error('lightweight chart load failed'));
        document.head.appendChild(script);
      });
    }
    return window.__bominalLwPromise;
  };

  const renderObservability = async () => {
    const [summaryResult, eventsResult, timeseriesResult, incidentsResult] = await Promise.all([
      requestJson('/api/admin/maintenance/metrics/summary'),
      requestJson('/api/admin/observability/events'),
      requestJson('/api/admin/observability/timeseries?window_minutes=240'),
      requestJson('/api/admin/incidents?limit=50'),
    ]);
    if (!summaryResult.ok) {
      content.innerHTML = `<div class="error-card">${escapeHtml(errorMessage(summaryResult))}</div>`;
      return;
    }
    if (!eventsResult.ok) {
      content.innerHTML = `<div class="error-card">${escapeHtml(errorMessage(eventsResult))}</div>`;
      return;
    }
    const summary = summaryResult.body || {};
    const events = Array.isArray(eventsResult.body?.events) ? eventsResult.body.events : [];
    const points = Array.isArray(timeseriesResult.body?.points) ? timeseriesResult.body.points : [];
    const incidents = incidentsResult.ok && Array.isArray(incidentsResult.body?.incidents) ? incidentsResult.body.incidents : [];
    const eventRows = events.slice(0, 80).map((event) => `
      <article class="summary-row">
        <span class="truncate">${escapeHtml(event.source)} · ${escapeHtml(event.event_type)}</span>
        <span class="text-xs txt-supporting">${escapeHtml(formatDate(event.occurred_at))}</span>
      </article>
    `).join('');
    const incidentRows = incidents.map((incident) => `
      <article class="admin-row" data-incident-id="${escapeHtml(incident.id)}">
        <div class="min-w-0">
          <p class="truncate text-sm font-semibold txt-strong">${escapeHtml(incident.title)}</p>
          <p class="mt-1 text-xs txt-supporting">${escapeHtml(incident.severity)} · ${escapeHtml(incident.status)} · ${escapeHtml(formatDate(incident.opened_at))}</p>
        </div>
        <div class="admin-row-actions">
          <button class="btn-ghost h-10 w-full md:w-auto" data-incident-status="monitoring">Monitoring</button>
          <button class="btn-ghost h-10 w-full md:w-auto" data-incident-status="resolved">Resolve</button>
        </div>
      </article>
    `).join('');
    content.innerHTML = `
      <section class="grid grid-cols-1 gap-2 md:grid-cols-2">
        <div class="summary-row"><span>Readiness</span><span class="badge">${summary.readiness_ok ? 'Healthy' : 'Degraded'}</span></div>
        <div class="summary-row"><span>Error rate</span><span>${asText(summary.error_rate, '0')}</span></div>
        <div class="summary-row"><span>P95 latency (ms)</span><span>${asText(summary.p95_latency_ms, 'n/a')}</span></div>
        <div class="summary-row"><span>Saturation</span><span>${asText(summary.saturation, 'n/a')}</span></div>
      </section>
      <section class="pt-2">
        <h2 class="text-base font-semibold txt-strong">Events over time</h2>
        <div id="obs-timeseries" class="mt-2 h-56 rounded-2xl border border-slate-200/70 bg-white/55 p-2"></div>
      </section>
      <section class="space-y-2 pt-2">
        <h2 class="text-base font-semibold txt-strong">Incident workflow</h2>
        <form id="incident-create-form" class="grid grid-cols-1 gap-2 md:grid-cols-2">
          <input class="field-input h-11" name="title" placeholder="Incident title" required minlength="3" maxlength="140" />
          <select class="field-input h-11" name="severity">
            <option value="sev1">sev1</option>
            <option value="sev2">sev2</option>
            <option value="sev3" selected>sev3</option>
            <option value="sev4">sev4</option>
          </select>
          <input class="field-input h-11 md:col-span-2" name="summary" placeholder="Summary (optional)" maxlength="600" />
          <input class="field-input h-11 md:col-span-2" name="reason" placeholder="Reason for opening incident" required minlength="8" />
          <button class="btn-primary h-11 w-full md:w-auto" type="submit">Open incident</button>
        </form>
        ${incidentRows || '<div class="empty-card">No incidents.</div>'}
      </section>
      <section class="space-y-2 pt-2">
        <h2 class="text-base font-semibold txt-strong">Events timeline</h2>
        ${eventRows || '<div class="empty-card">No observability events found.</div>'}
      </section>
    `;

    const chartEl = document.getElementById('obs-timeseries');
    if (chartEl && points.length) {
      try {
        const lw = await ensureLightweightCharts();
        if (lw && lw.createChart) {
          const chart = lw.createChart(chartEl, {
            width: chartEl.clientWidth || 320,
            height: 210,
            layout: {
              background: { color: 'transparent' },
              textColor: getComputedStyle(document.body).getPropertyValue('--text-supporting') ? `rgb(${getComputedStyle(document.body).getPropertyValue('--text-supporting')})` : '#64748b',
            },
            grid: {
              vertLines: { color: 'rgba(148,163,184,0.2)' },
              horzLines: { color: 'rgba(148,163,184,0.2)' },
            },
            rightPriceScale: { borderVisible: false },
            timeScale: { borderVisible: false, timeVisible: true, secondsVisible: false },
          });
          const totalSeries = chart.addLineSeries({ color: '#635bff', lineWidth: 2 });
          const errorSeries = chart.addLineSeries({ color: '#ef4444', lineWidth: 2 });
          totalSeries.setData(points.map((point) => ({ time: Math.floor(new Date(point.bucket).getTime() / 1000), value: Number(point.total_events || 0) })));
          errorSeries.setData(points.map((point) => ({ time: Math.floor(new Date(point.bucket).getTime() / 1000), value: Number(point.error_events || 0) })));
          chart.timeScale().fitContent();
          window.addEventListener('resize', () => {
            chart.applyOptions({ width: chartEl.clientWidth || 320 });
          });
        }
      } catch (_error) {
        chartEl.innerHTML = '<div class="empty-card">Chart unavailable in this environment.</div>';
      }
    }

    content.onclick = async (event) => {
      const button = event.target.closest('[data-incident-status]');
      if (!button) return;
      const row = button.closest('[data-incident-id]');
      const incidentId = row ? row.getAttribute('data-incident-id') : null;
      const nextStatus = button.getAttribute('data-incident-status');
      if (!incidentId || !nextStatus) return;
      const payload = await openConfirmModal({
        title: 'Update incident status',
        message: `Type ${incidentId} and provide a reason to update the incident status.`,
        targetLabel: incidentId,
        confirmText: 'Apply change',
      });
      if (!payload) return;
      const response = await requestJson(`/api/admin/incidents/${encodeURIComponent(incidentId)}/status`, 'PATCH', {
        status: nextStatus,
        reason: payload.reason,
        confirm_target: payload.confirm_target,
      });
      if (!response.ok) {
        setFlash('error', errorMessage(response));
        return;
      }
      setFlash('success', `Incident ${incidentId} updated.`);
      await renderObservability();
    };

    const form = document.getElementById('incident-create-form');
    if (form) {
      form.onsubmit = async (event) => {
        event.preventDefault();
        const formData = new FormData(form);
        const response = await requestJson('/api/admin/incidents', 'POST', {
          title: String(formData.get('title') || ''),
          severity: String(formData.get('severity') || 'sev3'),
          summary: String(formData.get('summary') || ''),
          reason: String(formData.get('reason') || ''),
        });
        if (!response.ok) {
          setFlash('error', errorMessage(response));
          return;
        }
        setFlash('success', 'Incident opened.');
        await renderObservability();
      };
    }
  };

  const renderSecurity = async () => {
    const [capabilitiesResult, sessionsResult] = await Promise.all([
      requestJson('/api/admin/capabilities'),
      requestJson('/api/admin/sessions'),
    ]);
    if (!capabilitiesResult.ok) {
      content.innerHTML = `<div class="error-card">${escapeHtml(errorMessage(capabilitiesResult))}</div>`;
      return;
    }
    if (!sessionsResult.ok) {
      content.innerHTML = `<div class="error-card">${escapeHtml(errorMessage(sessionsResult))}</div>`;
      return;
    }
    const capabilities = capabilitiesResult.body || {};
    const sessions = Array.isArray(sessionsResult.body?.sessions) ? sessionsResult.body.sessions : [];
    const sessionRows = sessions.slice(0, 24).map((session) => `
      <div class="summary-row">
        <span class="truncate">${escapeHtml(asText(session.email))}</span>
        <span class="text-xs txt-supporting">step-up: ${session.step_up_verified_at ? 'verified' : 'missing'}</span>
      </div>
    `).join('');
    content.innerHTML = `
      <section class="space-y-2">
        <h2 class="text-lg font-semibold txt-strong">Privileges</h2>
        <div class="summary-row"><span>Role</span><span>${escapeHtml(asText(capabilities.role))}</span></div>
        <div class="summary-row"><span>Can mutate</span><span>${capabilities.can_mutate ? 'yes' : 'no'}</span></div>
        <div class="summary-row"><span>Step-up required</span><span>${capabilities.step_up_required_for_mutation ? 'yes' : 'no'}</span></div>
      </section>
      <section class="space-y-2 pt-2">
        <h3 class="text-base font-semibold txt-strong">Session posture</h3>
        ${sessionRows || '<div class="empty-card">No sessions found.</div>'}
      </section>
      <section class="action-group action-single">
        <a class="btn-primary h-11 w-full text-center leading-[2.75rem]" href="/dashboard/security">Open account security</a>
      </section>
    `;
  };

  const renderConfig = async () => {
    const result = await requestJson('/api/admin/config/redacted');
    if (!result.ok) {
      content.innerHTML = `<div class="error-card">${escapeHtml(errorMessage(result))}</div>`;
      return;
    }
    const config = result.body?.config || {};
    const rows = Object.keys(config).sort().map((key) => `
      <div class="summary-row"><span class="truncate">${escapeHtml(key)}</span><span class="truncate text-xs txt-supporting">${escapeHtml(asText(config[key]))}</span></div>
    `).join('');
    content.innerHTML = `
      <section class="space-y-2">
        <h2 class="text-lg font-semibold txt-strong">Redacted configuration</h2>
        ${rows || '<div class="empty-card">No config keys available.</div>'}
      </section>
    `;
  };

  const renderAudit = async () => {
    const result = await requestJson('/api/admin/audit');
    if (!result.ok) {
      content.innerHTML = `<div class="error-card">${escapeHtml(errorMessage(result))}</div>`;
      return;
    }
    const entries = Array.isArray(result.body?.entries) ? result.body.entries : [];
    const rows = entries.slice(0, 120).map((entry) => `
      <article class="summary-card">
        <p class="text-sm font-semibold txt-strong">${escapeHtml(entry.action)} · ${escapeHtml(entry.target_type)}</p>
        <p class="mt-1 text-xs txt-supporting">${escapeHtml(entry.actor_email)} · ${escapeHtml(formatDate(entry.created_at))}</p>
        <p class="mt-1 text-xs txt-supporting">target: ${escapeHtml(asText(entry.target_id))}</p>
        <p class="mt-1 text-xs txt-faint">request_id: ${escapeHtml(asText(entry.request_id))}</p>
      </article>
    `).join('');
    content.innerHTML = `
      <section class="space-y-2">
        <h2 class="text-lg font-semibold txt-strong">Immutable admin audit</h2>
        ${rows || '<div class="empty-card">No audit records found.</div>'}
      </section>
    `;
  };

  const renderers = {
    users: renderUsers,
    runtime: renderRuntime,
    observability: renderObservability,
    security: renderSecurity,
    config: renderConfig,
    audit: renderAudit,
  };
  window.addEventListener('beforeunload', closeRuntimeJobsStream);
  if (section !== 'runtime') {
    closeRuntimeJobsStream();
  }
  const renderer = renderers[section];
  if (!renderer) {
    content.innerHTML = '<div class="error-card">Unsupported admin section.</div>';
    return;
  }
  renderer().catch((error) => {
    content.innerHTML = `<div class="error-card">${escapeHtml(String(error))}</div>`;
  });
})();
</script>
"##;

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
        let html = render_dashboard_security("admin@bominal.local");
        assert!(html.contains("data-action-group=\"destructive\""));
        assert!(html.contains("class=\"btn-destructive h-11 w-full\""));
        assert!(html.contains("const modalResult = await openSecurityModal({"));
        assert!(html.contains("title: 'Delete passkey'"));
    }

    #[test]
    fn admin_observability_chart_loader_uses_local_assets_path() {
        let html = render_admin_section("ops@bominal.com", "observability");
        assert!(html.contains("/assets/lightweight-charts.standalone.production.js"));
        assert!(!html.contains("cdn.jsdelivr.net/npm/lightweight-charts"));
    }

    #[test]
    fn admin_runtime_uses_sse_jobs_stream_endpoint() {
        let html = render_admin_section("ops@bominal.com", "runtime");
        assert!(html.contains("/api/admin/runtime/jobs/stream"));
    }
}
