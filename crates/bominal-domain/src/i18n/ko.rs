//! Korean (한국어) locale — default language.

use std::collections::HashMap;

pub fn messages() -> HashMap<&'static str, &'static str> {
    let mut m = HashMap::new();

    // Navigation
    m.insert("nav.home", "홈");
    m.insert("nav.search", "검색");
    m.insert("nav.tasks", "예약");
    m.insert("nav.reservations", "티켓");
    m.insert("nav.settings", "설정");

    // Auth
    m.insert("auth.login", "로그인");
    m.insert("auth.register", "회원가입");
    m.insert("auth.email", "이메일");
    m.insert("auth.password", "비밀번호");
    m.insert("auth.display_name", "이름");
    m.insert("auth.logout", "로그아웃");
    m.insert("auth.login_subtitle", "기차표 예매를 시작하세요");
    m.insert("auth.email_exists", "이미 등록된 이메일입니다");
    m.insert("auth.passkey_signin", "패스키로 로그인");
    m.insert("auth.welcome_back", "다시 오신 것을 환영합니다");
    m.insert(
        "auth.passkey_subtitle",
        "패스키 또는 비밀번호로 로그인하세요",
    );
    m.insert("auth.continue_email", "이메일로 계속");
    m.insert("auth.use_passkey", "패스키로 로그인");
    m.insert("auth.sign_in", "로그인");
    m.insert("auth.create_account", "계정 만들기");
    m.insert("auth.no_account", "계정이 없으신가요?");
    m.insert("auth.signup_link", "회원가입");
    m.insert("auth.has_account", "이미 계정이 있으신가요?");
    m.insert("auth.signin_link", "로그인");
    m.insert("auth.forgot_password", "비밀번호를 잊으셨나요?");
    m.insert("auth.reset_password", "비밀번호 재설정");
    m.insert("auth.reset_subtitle", "이메일로 재설정 링크를 보내드립니다");
    m.insert("auth.send_reset_link", "재설정 링크 보내기");
    m.insert("auth.back_to_signin", "로그인으로 돌아가기");
    m.insert("auth.back_to_signup", "회원가입으로 돌아가기");
    m.insert("auth.confirm_password", "비밀번호 확인");
    m.insert("auth.passwords_match", "비밀번호가 일치합니다");
    m.insert("auth.passwords_mismatch", "비밀번호가 일치하지 않습니다");
    m.insert("auth.pw_weak", "약함");
    m.insert("auth.pw_fair", "보통");
    m.insert("auth.pw_good", "양호");
    m.insert("auth.pw_strong", "강함");
    m.insert("auth.check_email", "이메일을 확인하세요");
    m.insert("auth.verify_sent_to", "인증 링크를 보냈습니다:");
    m.insert(
        "auth.verify_click_link",
        "링크를 클릭하여 계정을 확인하세요.",
    );
    m.insert("auth.resend_prompt", "받지 못하셨나요? 스팸함을 확인하거나");
    m.insert("auth.resend_link", "이메일 재전송");
    m.insert("auth.verified_continue", "이메일을 인증했습니다");
    m.insert("auth.add_passkey", "패스키를 추가하시겠습니까?");
    m.insert("auth.passkey_benefit_1", "비밀번호 없이 즉시 로그인");
    m.insert("auth.passkey_benefit_2", "기기 생체인증으로 보호");
    m.insert("auth.passkey_benefit_3", "모든 기기에서 동기화");
    m.insert("auth.add_passkey_now", "지금 패스키 추가");
    m.insert("auth.skip_for_now", "나중에 하기");
    m.insert("auth.get_started", "몇 초 만에 시작하세요");
    m.insert(
        "auth.enter_email_password",
        "이메일과 비밀번호를 입력하세요",
    );
    m.insert(
        "auth.reset_link_sent",
        "재설정 링크를 보냈습니다! 이메일을 확인하세요.",
    );
    m.insert("auth.verifying", "인증 중...");
    m.insert("auth.email_verified", "이메일 인증 완료");
    m.insert(
        "auth.email_verified_desc",
        "이메일이 성공적으로 인증되었습니다.",
    );
    m.insert("auth.verify_failed", "인증 실패");
    m.insert("auth.go_to_login", "로그인으로 돌아가기");
    m.insert("auth.new_password", "새 비밀번호");
    m.insert(
        "auth.password_reset_success",
        "비밀번호가 재설정되었습니다.",
    );
    m.insert("auth.missing_token", "유효하지 않은 링크입니다.");
    m.insert("auth.email_placeholder", "이메일 주소");

    // Search
    m.insert("search.departure", "출발역");
    m.insert("search.arrival", "도착역");
    m.insert("search.date", "날짜");
    m.insert("search.time", "시간");
    m.insert("search.passengers", "승객");
    m.insert("search.passenger_count", "명");
    m.insert("search.search_btn", "검색");
    m.insert("search.searching", "검색 중...");
    m.insert("search.edit", "수정");
    m.insert("search.no_results", "검색 결과가 없습니다");
    m.insert("search.auto_pay", "자동 결제");
    m.insert(
        "search.auto_pay_card_required",
        "자동 결제 작업에는 결제 카드가 필요합니다",
    );
    m.insert("search.notify", "알림");
    m.insert("search.auto_retry", "자동 재시도");

    // Task statuses
    m.insert("task.active", "진행 중");
    m.insert("task.completed", "완료");
    m.insert("task.queued", "대기");
    m.insert("task.running", "실행 중");
    m.insert("task.idle", "일시정지");
    m.insert("task.awaiting_payment", "결제 대기");
    m.insert("task.confirmed", "확정");
    m.insert("task.failed", "실패");
    m.insert("task.cancelled", "취소됨");
    m.insert("task.cancel", "취소");
    m.insert("task.cancelling", "취소 중...");
    m.insert("task.resume", "재개");
    m.insert("task.pause", "일시정지");
    m.insert("task.auto_retry", "자동 재시도");
    m.insert("task.created", "예약 작업이 생성되었습니다");

    // Settings
    m.insert("settings.title", "설정");
    m.insert("settings.section_provider", "예매 서비스");
    m.insert("settings.section_payment", "결제 수단");
    m.insert("settings.section_appearance", "화면 설정");
    m.insert("settings.section_security", "보안");
    m.insert("settings.section_notifications", "알림");
    m.insert("settings.theme", "테마");
    m.insert("settings.dark_mode", "다크 모드");
    m.insert("settings.light_mode", "라이트 모드");
    m.insert("settings.accessibility", "접근성");
    m.insert("settings.colorblind", "색각 이상 모드");
    m.insert("settings.language", "언어");
    m.insert("settings.theme_current", "기본");
    m.insert("settings.theme_transit_slate", "트랜짓 슬레이트");
    m.insert("settings.theme_night_teal", "나이트 틸");
    m.insert("settings.theme_warm_platform", "웜 플랫폼");

    // Provider
    m.insert("provider.srt", "SRT");
    m.insert("provider.ktx", "KTX/코레일");
    m.insert("provider.settings", "승차권 제공자 설정");
    m.insert("provider.login_id", "로그인 ID");
    m.insert("provider.password", "비밀번호");
    m.insert("provider.verify_save", "인증 및 저장");
    m.insert("provider.verifying", "인증 중...");
    m.insert("provider.remove", "삭제");
    m.insert("provider.status_valid", "유효");
    m.insert("provider.status_invalid", "유효하지 않음");
    m.insert("provider.status_unverified", "미인증");
    m.insert("provider.status_disabled", "비활성");
    m.insert("provider.not_configured", "미설정");
    m.insert("provider.credentials_required", "인증 정보가 필요합니다");
    m.insert("provider.invalid_auth", "인증 정보가 유효하지 않습니다");

    // Payment
    m.insert("payment.pay", "결제");
    m.insert("payment.paying", "결제 중...");
    m.insert("payment.card_label", "카드 이름");
    m.insert("payment.add_card", "카드 추가");
    m.insert("payment.card_number", "카드 번호");
    m.insert("payment.expiry", "유효기간");
    m.insert("payment.card_password", "카드 비밀번호 앞 2자리");
    m.insert("payment.birthday", "생년월일 (YYMMDD)");

    // Errors
    m.insert("error.network", "네트워크 오류가 발생했습니다");
    m.insert(
        "error.session_expired",
        "세션이 만료되었습니다. 다시 로그인해 주세요",
    );
    m.insert("error.sold_out", "매진되었습니다");
    m.insert("error.unexpected", "예상치 못한 오류가 발생했습니다");
    m.insert("error.login_failed", "로그인에 실패했습니다");
    m.insert("error.user_not_found", "존재하지 않는 회원입니다");
    m.insert("error.wrong_password", "비밀번호가 올바르지 않습니다");
    m.insert("error.no_remaining_seats", "잔여석이 없습니다");
    m.insert("error.standby_closed", "예약대기 접수가 마감되었습니다");
    m.insert("error.ip_blocked", "IP가 차단되었습니다");
    m.insert("error.passkey_failed", "패스키 로그인에 실패했습니다");
    m.insert(
        "error.passkey_register_failed",
        "패스키 등록에 실패했습니다",
    );

    // Common
    m.insert("common.confirm", "확인");
    m.insert("common.cancel", "취소");
    m.insert("common.save", "저장");
    m.insert("common.delete", "삭제");
    m.insert("common.loading", "로딩 중...");
    m.insert("common.retry", "재시도");
    m.insert("common.back", "뒤로");
    m.insert("common.next", "다음");
    m.insert("common.close", "닫기");
    m.insert("common.or", "또는");
    m.insert("common.coming_soon", "준비 중");

    // Search — extended
    m.insert("search.title", "열차 검색");
    m.insert("search.adults", "성인");
    m.insert("search.select_station", "역 선택");
    m.insert("search.tap_to_select", "열차를 선택하세요");
    m.insert("search.seat_preference", "좌석 유형");
    m.insert("search.seat_general_first", "일반석 우선");
    m.insert("search.seat_special_first", "특실 우선");
    m.insert("search.seat_general_only", "일반석만");
    m.insert("search.seat_special_only", "특실만");
    m.insert("search.create_task", "예약 시작");
    m.insert("search.creating_task", "예약 생성 중...");
    m.insert("search.no_cards", "등록된 카드가 없습니다");
    m.insert("search.add_card", "카드 추가");
    m.insert("search.select_card", "카드 선택");
    m.insert("search.view_tasks", "예약 목록 보기 →");

    // Reservation
    m.insert("reservation.title", "예약 내역");
    m.insert("reservation.paid", "결제 완료");
    m.insert("reservation.waiting", "예약 대기");
    m.insert("reservation.unpaid", "미결제");
    m.insert("reservation.cancel", "예약 취소");
    m.insert("reservation.cancelled", "예약이 취소되었습니다");
    m.insert("reservation.payment_success", "결제가 완료되었습니다");
    m.insert("reservation.no_active", "예약 내역이 없습니다");

    // Train types
    m.insert("train.ktx", "KTX");
    m.insert("train.srt", "SRT");
    m.insert("train.mugunghwa", "무궁화호");
    m.insert("train.itx_saemaeul", "ITX-새마을");

    // Review modal
    m.insert("review.title", "예약 확인");
    m.insert("review.priority_order", "우선순위 순서");
    m.insert("review.drag_reorder", "드래그하여 순서를 변경하세요");
    m.insert("review.start_reservation", "예약 시작");

    // Selection prompt
    m.insert("selection.selected_count", "개 선택됨");
    m.insert("selection.review", "확인하기");
    m.insert("selection.clear", "선택 해제");

    // Home
    m.insert("home.welcome", "환영합니다");
    m.insert("home.description", "기차표 예매를 도와드립니다");
    m.insert("home.quick_search", "빠른 검색");
    m.insert("home.active_tasks", "진행 중인 예약");
    m.insert("home.no_active_tasks", "진행 중인 예약이 없습니다");
    m.insert("home.quick_actions", "빠른 실행");
    m.insert("home.tickets", "승차권");
    m.insert("home.start_search", "열차 검색");
    m.insert("home.start_search_desc", "SRT · KTX 빈자리 찾기");
    m.insert("home.open_tasks", "예약 현황");
    m.insert("home.open_tasks_desc", "진행 중인 예약 확인");

    // Task — extended
    m.insert("task.no_active", "진행 중인 작업이 없습니다");
    m.insert("task.no_completed", "완료된 작업이 없습니다");
    m.insert("task.create_new", "새 예약 만들기");
    m.insert("task.attempts", "시도 횟수");
    m.insert("task.view_details", "상세 보기");
    m.insert("task.hide_details", "상세 숨기기");
    m.insert("task.notify", "알림");
    m.insert("task.retry", "재시도");
    m.insert("task.started_at", "시작 시간");
    m.insert("task.last_attempt", "마지막 시도");
    m.insert("task.not_started", "시작 전");
    m.insert("task.no_attempt", "시도 없음");
    m.insert("task.schedules_title", "예약 대상 열차");
    m.insert("task.total", "합계");
    m.insert("task.cancel_title", "작업 취소");
    m.insert("task.cancel_description", "이 작업을 취소하시겠습니까?");
    m.insert("task.cancel_confirm", "예, 취소");
    m.insert("task.keep", "유지");
    m.insert("task.pay_fare", "운임 결제");
    m.insert("task.seat_class", "좌석 유형");
    m.insert("task.passengers_label", "승객");

    // Seat labels
    m.insert("seat.general", "일반");
    m.insert("seat.special", "특실");

    // Passenger types
    m.insert("passenger.adult", "성인");
    m.insert("passenger.adult_desc", "만 13세 이상");
    m.insert("passenger.child", "어린이");
    m.insert("passenger.child_desc", "만 6~12세");
    m.insert("passenger.infant", "유아");
    m.insert("passenger.infant_desc", "만 6세 미만");
    m.insert("passenger.senior", "경로");
    m.insert("passenger.senior_desc", "만 65세 이상");
    m.insert("passenger.severe", "중증 장애");
    m.insert("passenger.severe_desc", "중증 장애인");
    m.insert("passenger.mild", "경증 장애");
    m.insert("passenger.mild_desc", "경증 장애인");
    m.insert("passenger.merit", "유공자");
    m.insert("passenger.merit_desc", "국가 유공자");
    m.insert("passenger.title", "승객 선택");
    m.insert("passenger.total", "합계");

    // Calendar/time modal
    m.insert("calendar.title", "날짜 및 시간");
    m.insert("calendar.apply", "적용");
    m.insert("calendar.month_1", "1월");
    m.insert("calendar.month_2", "2월");
    m.insert("calendar.month_3", "3월");
    m.insert("calendar.month_4", "4월");
    m.insert("calendar.month_5", "5월");
    m.insert("calendar.month_6", "6월");
    m.insert("calendar.month_7", "7월");
    m.insert("calendar.month_8", "8월");
    m.insert("calendar.month_9", "9월");
    m.insert("calendar.month_10", "10월");
    m.insert("calendar.month_11", "11월");
    m.insert("calendar.month_12", "12월");

    // Search — station labels
    m.insert("search.from", "출발");
    m.insert("search.to", "도착");

    // Review modal — extended
    m.insert("review.reorder_hint", "위아래 버튼으로 순서 변경");

    // Provider — extended
    m.insert("provider.setup", "설정");
    m.insert("provider.saved", "인증 정보가 저장되었습니다");

    // Payment — extended
    m.insert("payment.credit_card", "신용카드");
    m.insert("payment.debit_card", "체크카드");
    m.insert("payment.no_cards", "등록된 카드가 없습니다");

    // Error — extended
    m.insert("error.not_found", "페이지를 찾을 수 없습니다");
    m.insert("error.load_failed", "데이터를 불러오지 못했습니다");

    // Search — extra
    m.insert("search.go_to_search", "열차 검색하기");

    m
}
