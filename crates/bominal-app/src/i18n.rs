//! Internationalization module for the Bominal Train frontend.
//!
//! All user-visible text must be retrieved via [`t`].
//! Korean (ko) is the default locale. English (en) and Japanese (ja) are
//! supported. Locale detection reads `navigator.language`.
//!
//! # Key naming convention
//! - `common.*` — shared labels, buttons
//! - `auth.*` — authentication pages
//! - `error.*` — error messages

use std::collections::HashMap;
use std::sync::LazyLock;

/// Supported locales.
const DEFAULT_LOCALE: &str = "ko";

/// Per-locale translation tables: locale -> (key -> translated string).
static TRANSLATIONS: LazyLock<HashMap<&'static str, HashMap<&'static str, &'static str>>> =
    LazyLock::new(|| {
        let mut locales: HashMap<&str, HashMap<&str, &str>> = HashMap::new();
        locales.insert("ko", build_ko());
        locales.insert("en", build_en());
        locales.insert("ja", build_ja());
        locales
    });

/// Detect the user's preferred locale from the browser.
///
/// Reads `navigator.language` (e.g. "ja", "en-US", "ko-KR") and maps it
/// to a supported locale code. Falls back to [`DEFAULT_LOCALE`] when the
/// browser API is unavailable or the language is unsupported.
pub fn current_locale() -> &'static str {
    let lang = leptos::web_sys::window()
        .and_then(|w| w.navigator().language())
        .unwrap_or_default();

    let prefix = lang.split('-').next().unwrap_or("");
    match prefix {
        "en" => "en",
        "ja" => "ja",
        "ko" => "ko",
        _ => DEFAULT_LOCALE,
    }
}

/// Look up a translation key for the current locale.
/// Falls back to Korean, then returns the key itself if not found.
pub fn t(key: &str) -> &str {
    let locale = current_locale();
    TRANSLATIONS
        .get(locale)
        .and_then(|m| m.get(key).copied())
        .or_else(|| {
            TRANSLATIONS
                .get(DEFAULT_LOCALE)
                .and_then(|m| m.get(key).copied())
        })
        .unwrap_or(key)
}

// ---------------------------------------------------------------------------
// Korean translations
// ---------------------------------------------------------------------------

fn build_ko() -> HashMap<&'static str, &'static str> {
    let mut m = HashMap::new();

    // common
    m.insert("common.loading", "로딩 중...");
    m.insert("common.or", "또는");

    // auth
    m.insert("auth.get_started", "몇 초 만에 시작하세요");
    m.insert("auth.passkey_signin", "패스키로 로그인");
    m.insert("auth.continue_email", "이메일로 로그인");
    m.insert("auth.signup_link", "회원가입");
    m.insert("auth.welcome_back", "다시 오신 것을 환영합니다");
    m.insert("auth.enter_email_password", "이메일과 비밀번호를 입력하세요");
    m.insert("auth.email_placeholder", "이메일 주소");
    m.insert("auth.password", "비밀번호");
    m.insert("auth.show_password", "비밀번호 표시");
    m.insert("auth.hide_password", "비밀번호 숨기기");
    m.insert("auth.sign_in", "로그인");
    m.insert("auth.forgot_password", "비밀번호를 잊으셨나요?");
    m.insert("auth.create_account", "계정 만들기");
    m.insert("auth.display_name", "이름");
    m.insert("auth.has_account", "이미 계정이 있으신가요?");
    m.insert("auth.signin_link", "로그인");
    m.insert("auth.pw_weak", "약함");
    m.insert("auth.pw_fair", "보통");
    m.insert("auth.pw_good", "양호");
    m.insert("auth.pw_strong", "강함");
    m.insert("auth.reset_password", "비밀번호 재설정");
    m.insert("auth.reset_subtitle", "이메일로 재설정 링크를 보내드립니다");
    m.insert("auth.send_reset_link", "재설정 링크 보내기");
    m.insert("auth.back_to_signin", "로그인으로 돌아가기");
    m.insert("auth.reset_link_sent", "재설정 링크를 보냈습니다! 이메일을 확인하세요.");

    // error
    m.insert("error.passkey_failed", "패스키 로그인에 실패했습니다");
    m.insert("error.login_failed", "로그인에 실패했습니다");
    m.insert("error.unexpected", "예상치 못한 오류가 발생했습니다");
    m.insert("error.not_found", "페이지를 찾을 수 없습니다");
    m.insert("error.go_home", "홈으로 돌아가기");

    m
}

// ---------------------------------------------------------------------------
// English translations
// ---------------------------------------------------------------------------

fn build_en() -> HashMap<&'static str, &'static str> {
    let mut m = HashMap::new();

    // common
    m.insert("common.loading", "Loading...");
    m.insert("common.or", "or");

    // auth
    m.insert("auth.get_started", "Get started in seconds");
    m.insert("auth.passkey_signin", "Sign in with Passkey");
    m.insert("auth.continue_email", "Continue with Email");
    m.insert("auth.signup_link", "Sign up");
    m.insert("auth.welcome_back", "Welcome back");
    m.insert("auth.enter_email_password", "Enter your email and password");
    m.insert("auth.email_placeholder", "Email address");
    m.insert("auth.password", "Password");
    m.insert("auth.show_password", "Show password");
    m.insert("auth.hide_password", "Hide password");
    m.insert("auth.sign_in", "Sign In");
    m.insert("auth.forgot_password", "Forgot password?");
    m.insert("auth.create_account", "Create Account");
    m.insert("auth.display_name", "Name");
    m.insert("auth.has_account", "Already have an account?");
    m.insert("auth.signin_link", "Sign in");
    m.insert("auth.pw_weak", "Weak");
    m.insert("auth.pw_fair", "Fair");
    m.insert("auth.pw_good", "Good");
    m.insert("auth.pw_strong", "Strong");
    m.insert("auth.reset_password", "Reset password");
    m.insert("auth.reset_subtitle", "We'll send a reset link to your email");
    m.insert("auth.send_reset_link", "Send Reset Link");
    m.insert("auth.back_to_signin", "Back to Sign In");
    m.insert("auth.reset_link_sent", "Reset link sent! Check your email.");

    // error
    m.insert("error.passkey_failed", "Passkey login failed");
    m.insert("error.login_failed", "Login failed");
    m.insert("error.unexpected", "An unexpected error occurred");
    m.insert("error.not_found", "Page not found");
    m.insert("error.go_home", "Go home");

    m
}

// ---------------------------------------------------------------------------
// Japanese translations
// ---------------------------------------------------------------------------

fn build_ja() -> HashMap<&'static str, &'static str> {
    let mut m = HashMap::new();

    // common
    m.insert("common.loading", "読み込み中...");
    m.insert("common.or", "または");

    // auth
    m.insert("auth.get_started", "数秒で始められます");
    m.insert("auth.passkey_signin", "パスキーでサインイン");
    m.insert("auth.continue_email", "メールで続ける");
    m.insert("auth.signup_link", "新規登録");
    m.insert("auth.welcome_back", "おかえりなさい");
    m.insert("auth.enter_email_password", "メールアドレスとパスワードを入力");
    m.insert("auth.email_placeholder", "メールアドレス");
    m.insert("auth.password", "パスワード");
    m.insert("auth.show_password", "パスワードを表示");
    m.insert("auth.hide_password", "パスワードを隠す");
    m.insert("auth.sign_in", "サインイン");
    m.insert("auth.forgot_password", "パスワードをお忘れですか？");
    m.insert("auth.create_account", "アカウント作成");
    m.insert("auth.display_name", "名前");
    m.insert("auth.has_account", "既にアカウントをお持ちですか？");
    m.insert("auth.signin_link", "サインイン");
    m.insert("auth.pw_weak", "弱い");
    m.insert("auth.pw_fair", "普通");
    m.insert("auth.pw_good", "良好");
    m.insert("auth.pw_strong", "強い");
    m.insert("auth.reset_password", "パスワードリセット");
    m.insert("auth.reset_subtitle", "メールでリセットリンクをお送りします");
    m.insert("auth.send_reset_link", "リセットリンクを送信");
    m.insert("auth.back_to_signin", "サインインに戻る");
    m.insert("auth.reset_link_sent", "リセットリンクを送信しました！メールを確認してください。");

    // error
    m.insert("error.passkey_failed", "パスキーログインに失敗しました");
    m.insert("error.login_failed", "ログインに失敗しました");
    m.insert("error.unexpected", "予期しないエラーが発生しました");
    m.insert("error.not_found", "ページが見つかりません");
    m.insert("error.go_home", "ホームに戻る");

    m
}
