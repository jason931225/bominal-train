//! English locale.

use std::collections::HashMap;

pub fn messages() -> HashMap<&'static str, &'static str> {
    let mut m = HashMap::new();

    // Navigation
    m.insert("nav.home", "Home");
    m.insert("nav.search", "Search");
    m.insert("nav.tasks", "Tasks");
    m.insert("nav.reservations", "Tickets");
    m.insert("nav.settings", "Settings");

    // Auth
    m.insert("auth.login", "Log In");
    m.insert("auth.register", "Sign Up");
    m.insert("auth.email", "Email");
    m.insert("auth.password", "Password");
    m.insert("auth.display_name", "Name");
    m.insert("auth.logout", "Log Out");
    m.insert("auth.login_subtitle", "Start booking train tickets");
    m.insert("auth.email_exists", "Email already registered");
    m.insert("auth.passkey_signin", "Sign in with Passkey");
    m.insert("auth.welcome_back", "Welcome back");
    m.insert(
        "auth.passkey_subtitle",
        "Sign in with your passkey or password",
    );
    m.insert("auth.continue_email", "Continue with Email");
    m.insert("auth.use_passkey", "Use Passkey Instead");
    m.insert("auth.sign_in", "Sign In");
    m.insert("auth.create_account", "Create Account");
    m.insert("auth.no_account", "Don\u{2019}t have an account?");
    m.insert("auth.signup_link", "Sign up");
    m.insert("auth.has_account", "Already have an account?");
    m.insert("auth.signin_link", "Sign in");
    m.insert("auth.forgot_password", "Forgot password?");
    m.insert("auth.reset_password", "Reset password");
    m.insert(
        "auth.reset_subtitle",
        "We\u{2019}ll send a reset link to your email",
    );
    m.insert("auth.send_reset_link", "Send Reset Link");
    m.insert("auth.back_to_signin", "Back to Sign In");
    m.insert("auth.back_to_signup", "Back to sign up");
    m.insert("auth.confirm_password", "Confirm password");
    m.insert("auth.passwords_match", "Passwords match");
    m.insert("auth.passwords_mismatch", "Passwords do not match");
    m.insert("auth.pw_weak", "Weak");
    m.insert("auth.pw_fair", "Fair");
    m.insert("auth.pw_good", "Good");
    m.insert("auth.pw_strong", "Strong");
    m.insert("auth.check_email", "Check your email");
    m.insert("auth.verify_sent_to", "We sent a verification link to");
    m.insert(
        "auth.verify_click_link",
        "Click the link to confirm your account.",
    );
    m.insert("auth.resend_prompt", "Didn\u{2019}t get it? Check spam, or");
    m.insert("auth.resend_link", "resend the email");
    m.insert("auth.verified_continue", "I\u{2019}ve verified my email");
    m.insert("auth.add_passkey", "Add a Passkey?");
    m.insert(
        "auth.passkey_benefit_1",
        "Sign in instantly \u{2014} no passwords needed",
    );
    m.insert(
        "auth.passkey_benefit_2",
        "Protected by your device biometrics",
    );
    m.insert("auth.passkey_benefit_3", "Synced across all your devices");
    m.insert("auth.add_passkey_now", "Add Passkey Now");
    m.insert("auth.skip_for_now", "Skip for now");
    m.insert("auth.get_started", "Get started in seconds");
    m.insert("auth.enter_email_password", "Enter your email and password");
    m.insert("auth.reset_link_sent", "Reset link sent! Check your email.");
    m.insert("auth.verifying", "Verifying...");
    m.insert("auth.email_verified", "Email Verified");
    m.insert(
        "auth.email_verified_desc",
        "Your email has been verified successfully.",
    );
    m.insert("auth.verify_failed", "Verification Failed");
    m.insert("auth.go_to_login", "Go to Login");
    m.insert("auth.new_password", "New password");
    m.insert(
        "auth.password_reset_success",
        "Password has been reset successfully.",
    );
    m.insert("auth.missing_token", "Invalid or missing link.");
    m.insert("auth.email_placeholder", "Email address");

    // Search
    m.insert("search.departure", "Departure");
    m.insert("search.arrival", "Arrival");
    m.insert("search.date", "Date");
    m.insert("search.time", "Time");
    m.insert("search.passengers", "Passengers");
    m.insert("search.passenger_count", "pax");
    m.insert("search.search_btn", "Search");
    m.insert("search.searching", "Searching...");
    m.insert("search.edit", "Edit");
    m.insert("search.no_results", "No results found");
    m.insert("search.auto_pay", "Auto Pay");
    m.insert(
        "search.auto_pay_card_required",
        "A payment card is required for auto-pay tasks",
    );
    m.insert("search.notify", "Notify");
    m.insert("search.auto_retry", "Auto Retry");

    // Task statuses
    m.insert("task.active", "Active");
    m.insert("task.completed", "Completed");
    m.insert("task.queued", "Queued");
    m.insert("task.running", "Running");
    m.insert("task.idle", "Paused");
    m.insert("task.awaiting_payment", "Awaiting Payment");
    m.insert("task.confirmed", "Confirmed");
    m.insert("task.failed", "Failed");
    m.insert("task.cancelled", "Cancelled");
    m.insert("task.cancel", "Cancel");
    m.insert("task.cancelling", "Cancelling...");
    m.insert("task.resume", "Resume");
    m.insert("task.pause", "Pause");
    m.insert("task.auto_retry", "Auto Retry");
    m.insert("task.created", "Reservation task created!");

    // Settings
    m.insert("settings.title", "Settings");
    m.insert("settings.section_provider", "Train Services");
    m.insert("settings.section_payment", "Payment");
    m.insert("settings.section_appearance", "Appearance");
    m.insert("settings.section_security", "Security");
    m.insert("settings.section_notifications", "Notifications");
    m.insert("settings.theme", "Theme");
    m.insert("settings.dark_mode", "Dark Mode");
    m.insert("settings.light_mode", "Light Mode");
    m.insert("settings.accessibility", "Accessibility");
    m.insert("settings.colorblind", "Colorblind Mode");
    m.insert("settings.language", "Language");
    m.insert("settings.theme_current", "Default");
    m.insert("settings.theme_transit_slate", "Transit Slate");
    m.insert("settings.theme_night_teal", "Night Teal");
    m.insert("settings.theme_warm_platform", "Warm Platform");

    // Provider
    m.insert("provider.srt", "SRT");
    m.insert("provider.ktx", "KTX/Korail");
    m.insert("provider.settings", "Provider Settings");
    m.insert("provider.login_id", "Login ID");
    m.insert("provider.password", "Password");
    m.insert("provider.verify_save", "Verify & Save");
    m.insert("provider.verifying", "Verifying...");
    m.insert("provider.remove", "Remove");
    m.insert("provider.status_valid", "Valid");
    m.insert("provider.status_invalid", "Invalid");
    m.insert("provider.status_unverified", "Unverified");
    m.insert("provider.status_disabled", "Disabled");
    m.insert("provider.not_configured", "Not configured");
    m.insert(
        "provider.credentials_required",
        "Provider credentials required",
    );
    m.insert("provider.invalid_auth", "Provider credentials are invalid");

    // Payment
    m.insert("payment.pay", "Pay");
    m.insert("payment.paying", "Paying...");
    m.insert("payment.card_label", "Card Name");
    m.insert("payment.add_card", "Add Card");
    m.insert("payment.card_number", "Card Number");
    m.insert("payment.expiry", "Expiry");
    m.insert("payment.card_password", "First 2 digits of card PIN");
    m.insert("payment.birthday", "Birthday (YYMMDD)");

    // Errors
    m.insert("error.network", "A network error occurred");
    m.insert(
        "error.session_expired",
        "Session expired. Please log in again",
    );
    m.insert("error.sold_out", "Sold out");
    m.insert("error.unexpected", "An unexpected error occurred");
    m.insert("error.login_failed", "Login failed");
    m.insert("error.user_not_found", "User not found");
    m.insert("error.wrong_password", "Incorrect password");
    m.insert("error.no_remaining_seats", "No remaining seats");
    m.insert("error.standby_closed", "Standby registration is closed");
    m.insert("error.ip_blocked", "Your IP has been blocked");
    m.insert("error.passkey_failed", "Passkey login failed");

    // Common
    m.insert("common.confirm", "Confirm");
    m.insert("common.cancel", "Cancel");
    m.insert("common.save", "Save");
    m.insert("common.delete", "Delete");
    m.insert("common.loading", "Loading...");
    m.insert("common.retry", "Retry");
    m.insert("common.back", "Back");
    m.insert("common.next", "Next");
    m.insert("common.close", "Close");
    m.insert("common.or", "or");
    m.insert("common.coming_soon", "Coming soon");

    // Search — extended
    m.insert("search.title", "Search Trains");
    m.insert("search.adults", "Adults");
    m.insert("search.select_station", "Select station");
    m.insert("search.tap_to_select", "Tap to select trains");
    m.insert("search.seat_preference", "Seat Preference");
    m.insert("search.seat_general_first", "General First");
    m.insert("search.seat_special_first", "Special First");
    m.insert("search.seat_general_only", "General Only");
    m.insert("search.seat_special_only", "Special Only");
    m.insert("search.create_task", "Start Reservation Task");
    m.insert("search.creating_task", "Creating task...");
    m.insert("search.no_cards", "No cards added.");
    m.insert("search.add_card", "Add a card");
    m.insert("search.select_card", "Select card");
    m.insert("search.view_tasks", "View tasks →");

    // Reservation
    m.insert("reservation.title", "Reservations");
    m.insert("reservation.paid", "Paid");
    m.insert("reservation.waiting", "Standby");
    m.insert("reservation.unpaid", "Unpaid");
    m.insert("reservation.cancel", "Cancel Reservation");
    m.insert("reservation.cancelled", "Reservation cancelled");
    m.insert("reservation.payment_success", "Payment successful");
    m.insert("reservation.no_active", "No active reservations");

    // Train types
    m.insert("train.ktx", "KTX");
    m.insert("train.srt", "SRT");
    m.insert("train.mugunghwa", "Mugunghwa");
    m.insert("train.itx_saemaeul", "ITX-Saemaeul");

    // Review modal
    m.insert("review.title", "Review Reservation");
    m.insert("review.priority_order", "Priority Order");
    m.insert("review.drag_reorder", "Drag to reorder");
    m.insert("review.start_reservation", "Start Reservation");

    // Selection prompt
    m.insert("selection.selected_count", "selected");
    m.insert("selection.review", "Review");
    m.insert("selection.clear", "Clear");

    // Home
    m.insert("home.welcome", "Welcome");
    m.insert("home.quick_search", "Quick Search");
    m.insert("home.active_tasks", "Active Tasks");
    m.insert("home.no_active_tasks", "No active tasks");
    m.insert("home.quick_actions", "Quick Actions");
    m.insert("home.tickets", "Tickets");

    // Task — extended
    m.insert("task.no_active", "No active tasks");
    m.insert("task.no_completed", "No completed tasks");
    m.insert("task.create_new", "Create a new task");
    m.insert("task.attempts", "Attempts");
    m.insert("task.view_details", "View Details");
    m.insert("task.hide_details", "Hide Details");
    m.insert("task.notify", "Notify");
    m.insert("task.retry", "Retry");
    m.insert("task.started_at", "Started at");
    m.insert("task.last_attempt", "Last attempt");
    m.insert("task.not_started", "Not started");
    m.insert("task.no_attempt", "No attempt yet");
    m.insert("task.schedules_title", "Target Trains");
    m.insert("task.total", "total");
    m.insert("task.cancel_title", "Cancel Task?");
    m.insert(
        "task.cancel_description",
        "Are you sure you want to cancel this task?",
    );
    m.insert("task.cancel_confirm", "Yes, Cancel Task");
    m.insert("task.keep", "Keep Task");
    m.insert("task.pay_fare", "Pay Fare");
    m.insert("task.seat_class", "Seat Class");
    m.insert("task.passengers_label", "Passengers");

    // Seat labels
    m.insert("seat.general", "General");
    m.insert("seat.special", "Special");

    // Passenger types
    m.insert("passenger.adult", "Adult");
    m.insert("passenger.adult_desc", "13 or older");
    m.insert("passenger.child", "Child");
    m.insert("passenger.child_desc", "6 to 12");
    m.insert("passenger.infant", "Infant");
    m.insert("passenger.infant_desc", "Below 6");
    m.insert("passenger.senior", "Senior");
    m.insert("passenger.senior_desc", "65 or older");
    m.insert("passenger.severe", "Severe Disability");
    m.insert("passenger.severe_desc", "Severely disabled");
    m.insert("passenger.mild", "Mild Disability");
    m.insert("passenger.mild_desc", "Mildly disabled");
    m.insert("passenger.merit", "Merit");
    m.insert("passenger.merit_desc", "Person of merit");
    m.insert("passenger.title", "Select Passengers");
    m.insert("passenger.total", "Total");

    // Calendar/time modal
    m.insert("calendar.title", "Date & Time");
    m.insert("calendar.apply", "Apply");

    // Search — station labels
    m.insert("search.from", "From");
    m.insert("search.to", "To");

    // Review modal — extended
    m.insert("review.reorder_hint", "Reorder with arrows");

    // Provider — extended
    m.insert("provider.setup", "Setup");
    m.insert("provider.saved", "Credentials verified and saved");

    // Payment — extended
    m.insert("payment.credit_card", "Credit Card");
    m.insert("payment.debit_card", "Debit Card");
    m.insert("payment.no_cards", "No cards added");

    // Error — extended
    m.insert("error.not_found", "Page not found");
    m.insert("error.load_failed", "Failed to load data");

    // Search — extra
    m.insert("search.go_to_search", "Search for trains");

    m
}
