//! Japanese (日本語) locale.

use std::collections::HashMap;

pub fn messages() -> HashMap<&'static str, &'static str> {
    let mut m = HashMap::new();

    // Navigation
    m.insert("nav.home", "ホーム");
    m.insert("nav.search", "検索");
    m.insert("nav.tasks", "予約");
    m.insert("nav.settings", "設定");

    // Auth
    m.insert("auth.login", "ログイン");
    m.insert("auth.register", "新規登録");
    m.insert("auth.email", "メールアドレス");
    m.insert("auth.password", "パスワード");
    m.insert("auth.display_name", "名前");
    m.insert("auth.logout", "ログアウト");
    m.insert("auth.login_subtitle", "列車チケットの予約を始めましょう");
    m.insert("auth.email_exists", "既に登録されたメールアドレスです");

    // Search
    m.insert("search.departure", "出発駅");
    m.insert("search.arrival", "到着駅");
    m.insert("search.date", "日付");
    m.insert("search.time", "時間");
    m.insert("search.passengers", "乗客");
    m.insert("search.passenger_count", "名");
    m.insert("search.search_btn", "検索");
    m.insert("search.searching", "検索中...");
    m.insert("search.edit", "編集");
    m.insert("search.no_results", "検索結果がありません");
    m.insert("search.auto_pay", "自動決済");
    m.insert("search.notify", "通知");
    m.insert("search.auto_retry", "自動リトライ");

    // Task statuses
    m.insert("task.active", "進行中");
    m.insert("task.completed", "完了");
    m.insert("task.queued", "待機中");
    m.insert("task.running", "実行中");
    m.insert("task.idle", "一時停止");
    m.insert("task.awaiting_payment", "決済待ち");
    m.insert("task.confirmed", "確定");
    m.insert("task.failed", "失敗");
    m.insert("task.cancelled", "キャンセル済み");
    m.insert("task.cancel", "キャンセル");
    m.insert("task.cancelling", "キャンセル中...");
    m.insert("task.resume", "再開");
    m.insert("task.pause", "一時停止");
    m.insert("task.created", "予約タスクが作成されました");

    // Settings
    m.insert("settings.title", "設定");
    m.insert("settings.theme", "テーマ");
    m.insert("settings.dark_mode", "ダークモード");
    m.insert("settings.light_mode", "ライトモード");
    m.insert("settings.accessibility", "アクセシビリティ");
    m.insert("settings.colorblind", "色覚サポートモード");
    m.insert("settings.language", "言語");
    m.insert("settings.theme_current", "デフォルト");
    m.insert("settings.theme_transit_slate", "トランジットスレート");
    m.insert("settings.theme_night_teal", "ナイトティール");
    m.insert("settings.theme_warm_platform", "ウォームプラットフォーム");

    // Provider
    m.insert("provider.srt", "SRT");
    m.insert("provider.ktx", "KTX/コレール");
    m.insert("provider.settings", "鉄道会社設定");
    m.insert("provider.login_id", "ログインID");
    m.insert("provider.password", "パスワード");
    m.insert("provider.verify_save", "認証して保存");
    m.insert("provider.verifying", "認証中...");
    m.insert("provider.remove", "削除");
    m.insert("provider.status_valid", "有効");
    m.insert("provider.status_invalid", "無効");
    m.insert("provider.status_unverified", "未認証");
    m.insert("provider.status_disabled", "無効化");
    m.insert("provider.not_configured", "未設定");
    m.insert("provider.credentials_required", "認証情報が必要です");
    m.insert("provider.invalid_auth", "認証情報が無効です");

    // Payment
    m.insert("payment.pay", "決済");
    m.insert("payment.paying", "決済中...");
    m.insert("payment.card_label", "カード名");
    m.insert("payment.add_card", "カード追加");
    m.insert("payment.card_number", "カード番号");
    m.insert("payment.expiry", "有効期限");
    m.insert("payment.card_password", "暗証番号の最初の2桁");
    m.insert("payment.birthday", "生年月日 (YYMMDD)");

    // Errors
    m.insert("error.network", "ネットワークエラーが発生しました");
    m.insert("error.session_expired", "セッションが期限切れです。再度ログインしてください");
    m.insert("error.sold_out", "売り切れ");
    m.insert("error.unexpected", "予期しないエラーが発生しました");
    m.insert("error.login_failed", "ログインに失敗しました");
    m.insert("error.user_not_found", "ユーザーが見つかりません");
    m.insert("error.wrong_password", "パスワードが正しくありません");
    m.insert("error.no_remaining_seats", "残席がありません");
    m.insert("error.standby_closed", "キャンセル待ち受付は終了しました");
    m.insert("error.ip_blocked", "IPがブロックされています");

    // Common
    m.insert("common.confirm", "確認");
    m.insert("common.cancel", "キャンセル");
    m.insert("common.save", "保存");
    m.insert("common.delete", "削除");
    m.insert("common.loading", "読み込み中...");
    m.insert("common.retry", "リトライ");
    m.insert("common.back", "戻る");
    m.insert("common.next", "次へ");
    m.insert("common.close", "閉じる");

    // Search — extended
    m.insert("search.title", "列車検索");
    m.insert("search.adults", "大人");
    m.insert("search.select_station", "駅を選択");
    m.insert("search.tap_to_select", "列車を選択してください");
    m.insert("search.seat_preference", "座席タイプ");
    m.insert("search.seat_general_first", "一般席優先");
    m.insert("search.seat_special_first", "特室優先");
    m.insert("search.seat_general_only", "一般席のみ");
    m.insert("search.seat_special_only", "特室のみ");
    m.insert("search.create_task", "予約タスク開始");
    m.insert("search.creating_task", "タスク作成中...");
    m.insert("search.no_cards", "カードが登録されていません");
    m.insert("search.add_card", "カードを追加");
    m.insert("search.select_card", "カードを選択");
    m.insert("search.view_tasks", "予約一覧を見る →");

    // Reservation
    m.insert("reservation.title", "予約一覧");
    m.insert("reservation.paid", "決済済み");
    m.insert("reservation.waiting", "待機中");
    m.insert("reservation.unpaid", "未決済");
    m.insert("reservation.cancel", "予約キャンセル");
    m.insert("reservation.cancelled", "予約がキャンセルされました");
    m.insert("reservation.payment_success", "決済が完了しました");
    m.insert("reservation.no_active", "予約がありません");

    // Train types
    m.insert("train.ktx", "KTX");
    m.insert("train.srt", "SRT");
    m.insert("train.mugunghwa", "ムグンファ号");
    m.insert("train.itx_saemaeul", "ITX-セマウル");

    // Review modal
    m.insert("review.title", "予約確認");
    m.insert("review.priority_order", "優先順位");
    m.insert("review.drag_reorder", "ドラッグして並べ替え");
    m.insert("review.start_reservation", "予約開始");

    // Selection prompt
    m.insert("selection.selected_count", "件選択中");
    m.insert("selection.review", "確認する");
    m.insert("selection.clear", "選択解除");

    // Home
    m.insert("home.welcome", "ようこそ");
    m.insert("home.quick_search", "クイック検索");
    m.insert("home.active_tasks", "進行中の予約");
    m.insert("home.no_active_tasks", "進行中の予約はありません");
    m.insert("home.quick_actions", "クイック操作");
    m.insert("home.tickets", "乗車券");

    // Task — extended
    m.insert("task.no_active", "進行中のタスクはありません");
    m.insert("task.no_completed", "完了したタスクはありません");
    m.insert("task.create_new", "新しい予約を作成");
    m.insert("task.attempts", "試行回数");

    // Seat labels
    m.insert("seat.general", "一般");
    m.insert("seat.special", "特室");

    // Provider — extended
    m.insert("provider.setup", "設定");
    m.insert("provider.saved", "認証情報が保存されました");

    // Payment — extended
    m.insert("payment.credit_card", "クレジットカード");
    m.insert("payment.debit_card", "デビットカード");
    m.insert("payment.no_cards", "登録されたカードがありません");

    // Error — extended
    m.insert("error.not_found", "ページが見つかりません");
    m.insert("error.load_failed", "データの読み込みに失敗しました");

    // Search — extra
    m.insert("search.go_to_search", "列車を検索する");

    m
}
