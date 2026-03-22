//! Japanese (日本語) locale.

use std::collections::HashMap;

pub fn messages() -> HashMap<&'static str, &'static str> {
    let mut m = HashMap::new();

    // Navigation
    m.insert("nav.home", "ホーム");
    m.insert("nav.search", "検索");
    m.insert("nav.tasks", "予約");
    m.insert("nav.reservations", "チケット");
    m.insert("nav.settings", "設定");

    // Auth
    m.insert("auth.login", "ログイン");
    m.insert("auth.register", "新規登録");
    m.insert("auth.email", "メールアドレス");
    m.insert("auth.password", "パスワード");
    m.insert("auth.show_password", "パスワードを表示");
    m.insert("auth.hide_password", "パスワードを隠す");
    m.insert("auth.display_name", "名前");
    m.insert("auth.logout", "ログアウト");
    m.insert("auth.login_subtitle", "列車チケットの予約を始めましょう");
    m.insert("auth.email_exists", "既に登録されたメールアドレスです");
    m.insert("auth.passkey_signin", "パスキーでサインイン");
    m.insert("auth.welcome_back", "おかえりなさい");
    m.insert(
        "auth.passkey_subtitle",
        "パスキーまたはパスワードでサインイン",
    );
    m.insert("auth.continue_email", "メールで続ける");
    m.insert("auth.use_passkey", "パスキーを使用");
    m.insert("auth.sign_in", "サインイン");
    m.insert("auth.create_account", "アカウント作成");
    m.insert("auth.no_account", "アカウントをお持ちでないですか？");
    m.insert("auth.signup_link", "新規登録");
    m.insert("auth.has_account", "既にアカウントをお持ちですか？");
    m.insert("auth.signin_link", "サインイン");
    m.insert("auth.forgot_password", "パスワードをお忘れですか？");
    m.insert("auth.reset_password", "パスワードリセット");
    m.insert(
        "auth.reset_subtitle",
        "メールでリセットリンクをお送りします",
    );
    m.insert("auth.send_reset_link", "リセットリンクを送信");
    m.insert("auth.back_to_signin", "サインインに戻る");
    m.insert("auth.back_to_signup", "新規登録に戻る");
    m.insert("auth.confirm_password", "パスワード確認");
    m.insert("auth.passwords_match", "パスワードが一致しています");
    m.insert("auth.passwords_mismatch", "パスワードが一致しません");
    m.insert("auth.pw_weak", "弱い");
    m.insert("auth.pw_fair", "普通");
    m.insert("auth.pw_good", "良好");
    m.insert("auth.pw_strong", "強い");
    m.insert("auth.check_email", "メールを確認してください");
    m.insert("auth.verify_sent_to", "確認リンクを送信しました：");
    m.insert(
        "auth.verify_click_link",
        "リンクをクリックしてアカウントを確認してください。",
    );
    m.insert("auth.resend_prompt", "届きませんか？迷惑メールを確認するか");
    m.insert("auth.resend_link", "メールを再送信");
    m.insert("auth.verified_continue", "メールを認証しました");
    m.insert("auth.add_passkey", "パスキーを追加しますか？");
    m.insert("auth.passkey_benefit_1", "パスワード不要で即座にサインイン");
    m.insert("auth.passkey_benefit_2", "デバイスの生体認証で保護");
    m.insert("auth.passkey_benefit_3", "すべてのデバイスで同期");
    m.insert("auth.add_passkey_now", "今すぐパスキーを追加");
    m.insert("auth.skip_for_now", "後で設定する");
    m.insert("auth.get_started", "数秒で始められます");
    m.insert(
        "auth.enter_email_password",
        "メールアドレスとパスワードを入力",
    );
    m.insert(
        "auth.reset_link_sent",
        "リセットリンクを送信しました！メールを確認してください。",
    );
    m.insert("auth.verifying", "認証中...");
    m.insert("auth.email_verified", "メール認証完了");
    m.insert("auth.email_verified_desc", "メールが正常に認証されました。");
    m.insert("auth.verify_failed", "認証失敗");
    m.insert("auth.go_to_login", "ログインに戻る");
    m.insert("auth.new_password", "新しいパスワード");
    m.insert(
        "auth.password_reset_success",
        "パスワードが再設定されました。",
    );
    m.insert("auth.missing_token", "無効なリンクです。");
    m.insert("auth.email_placeholder", "メールアドレス");

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
    m.insert(
        "search.auto_pay_card_required",
        "自動決済タスクには支払いカードが必要です",
    );
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
    m.insert("task.auto_retry", "自動リトライ");
    m.insert("task.created", "予約タスクが作成されました");

    // Settings
    m.insert("settings.title", "設定");
    m.insert("settings.section_provider", "予約サービス");
    m.insert("settings.section_payment", "決済手段");
    m.insert("settings.section_appearance", "表示設定");
    m.insert("settings.section_security", "セキュリティ");
    m.insert("settings.section_notifications", "通知");
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
    m.insert(
        "error.session_expired",
        "セッションが期限切れです。再度ログインしてください",
    );
    m.insert("error.sold_out", "売り切れ");
    m.insert("error.unexpected", "予期しないエラーが発生しました");
    m.insert("error.login_failed", "ログインに失敗しました");
    m.insert("error.user_not_found", "ユーザーが見つかりません");
    m.insert("error.wrong_password", "パスワードが正しくありません");
    m.insert("error.no_remaining_seats", "残席がありません");
    m.insert("error.standby_closed", "キャンセル待ち受付は終了しました");
    m.insert("error.ip_blocked", "IPがブロックされています");
    m.insert("error.passkey_failed", "パスキーログインに失敗しました");
    m.insert(
        "error.passkey_register_failed",
        "パスキー登録に失敗しました",
    );

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
    m.insert("common.or", "または");
    m.insert("common.coming_soon", "準備中");

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
    m.insert("home.description", "列車の予約をお手伝いします");
    m.insert("home.quick_search", "クイック検索");
    m.insert("home.active_tasks", "進行中の予約");
    m.insert("home.no_active_tasks", "進行中の予約はありません");
    m.insert("home.quick_actions", "クイック操作");
    m.insert("home.tickets", "乗車券");
    m.insert("home.start_search", "列車検索");
    m.insert("home.start_search_desc", "SRT・KTX空席検索");
    m.insert("home.open_tasks", "予約状況");
    m.insert("home.open_tasks_desc", "進行中の予約を確認");

    // Task — extended
    m.insert("task.no_active", "進行中のタスクはありません");
    m.insert("task.no_completed", "完了したタスクはありません");
    m.insert("task.create_new", "新しい予約を作成");
    m.insert("task.attempts", "試行回数");
    m.insert("task.view_details", "詳細を見る");
    m.insert("task.hide_details", "詳細を隠す");
    m.insert("task.notify", "通知");
    m.insert("task.retry", "リトライ");
    m.insert("task.started_at", "開始時間");
    m.insert("task.last_attempt", "最終試行");
    m.insert("task.not_started", "未開始");
    m.insert("task.no_attempt", "試行なし");
    m.insert("task.schedules_title", "対象列車");
    m.insert("task.total", "合計");
    m.insert("task.cancel_title", "タスク取消");
    m.insert("task.cancel_description", "このタスクを取り消しますか？");
    m.insert("task.cancel_confirm", "はい、取消");
    m.insert("task.keep", "維持");
    m.insert("task.pay_fare", "運賃支払");
    m.insert("task.seat_class", "座席クラス");
    m.insert("task.passengers_label", "乗客");

    // Seat labels
    m.insert("seat.general", "一般");
    m.insert("seat.special", "特室");

    // Passenger types
    m.insert("passenger.adult", "大人");
    m.insert("passenger.adult_desc", "13歳以上");
    m.insert("passenger.child", "子供");
    m.insert("passenger.child_desc", "6〜12歳");
    m.insert("passenger.infant", "幼児");
    m.insert("passenger.infant_desc", "6歳未満");
    m.insert("passenger.senior", "シニア");
    m.insert("passenger.senior_desc", "65歳以上");
    m.insert("passenger.severe", "重度障害");
    m.insert("passenger.severe_desc", "重度障害者");
    m.insert("passenger.mild", "軽度障害");
    m.insert("passenger.mild_desc", "軽度障害者");
    m.insert("passenger.merit", "有功者");
    m.insert("passenger.merit_desc", "国家有功者");
    m.insert("passenger.title", "乗客選択");
    m.insert("passenger.total", "合計");

    // Calendar/time modal
    m.insert("calendar.title", "日時選択");
    m.insert("calendar.apply", "適用");
    m.insert("calendar.prev_month", "前月");
    m.insert("calendar.next_month", "翌月");
    m.insert("calendar.month_1", "1月");
    m.insert("calendar.month_2", "2月");
    m.insert("calendar.month_3", "3月");
    m.insert("calendar.month_4", "4月");
    m.insert("calendar.month_5", "5月");
    m.insert("calendar.month_6", "6月");
    m.insert("calendar.month_7", "7月");
    m.insert("calendar.month_8", "8月");
    m.insert("calendar.month_9", "9月");
    m.insert("calendar.month_10", "10月");
    m.insert("calendar.month_11", "11月");
    m.insert("calendar.month_12", "12月");

    // Search — station labels
    m.insert("search.from", "出発");
    m.insert("search.to", "到着");
    m.insert("search.swap_stations", "駅を入れ替え");
    m.insert("search.provider", "鉄道会社");

    // Passenger counter
    m.insert("passenger.decrease", "減らす");
    m.insert("passenger.increase", "増やす");

    // Review modal — extended
    m.insert("review.reorder_hint", "矢印で順序変更");

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
