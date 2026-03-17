//! DynaPath token generation for KTX/Korail API.
//!
//! Faithfully ported from `ktx.py:68-177` (DynaPathMasterEngine class).
//!
//! The algorithm uses a custom encoding scheme (NOT standard base64/UTF-8)
//! with a factorial number system (Lehmer code) permutation table.

use rand::Rng;

/// Base table for DynaPath encoding (62 chars).
const TABLE: &str = "3FE9jgRD4KdCyuawklqGJYmvfMn15P7US8XbxeLQtWT6OicBAopINs2Vh0HZrz";

/// Encoding parameters.
const I8: u64 = 161;
const I9: usize = 30;
const I10: usize = 2;

/// DynaPath engine constants.
const APP_ID: &str = "com.korail.talk";
const AS_VALUE: &str = "%5B38ff229cb34c7dda8e28220a2d750cce%5D";
const DEVICE_MODEL: &str = "SM-S928N";
const OS_TYPE: &str = "Android";
const SDK_VERSION: &str = "v1";

/// Paths that require DynaPath token (substring match).
const REQUIRED_PATHS: &[&str] = &[
    "TicketReservation",
    "NonMemTicket",
    "ScheduleView",
    "ScheduleViewSpecial",
    "trn.prcFare.do",
    "login.Login",
];

/// Check if a URL path requires a DynaPath token.
pub fn requires_token(path: &str) -> bool {
    REQUIRED_PATHS.iter().any(|p| path.contains(p))
}

/// DynaPath token engine.
///
/// `app_start_ts` is set ONCE at engine creation (not per call).
pub struct DynaPathEngine {
    app_start_ts: String,
}

impl DynaPathEngine {
    /// Create a new engine. `app_start_ts` is set to current time in milliseconds.
    pub fn new(app_start_ts_ms: u64) -> Self {
        Self {
            app_start_ts: app_start_ts_ms.to_string(),
        }
    }

    /// Generate a DynaPath token for a request.
    ///
    /// - `device_id`: device identifier (e.g. "558a4f02041657ea")
    /// - `ts`: current timestamp in milliseconds
    /// - `rand_str`: 4-char random alphanumeric string (uppercase + digits)
    pub fn generate_token(&self, device_id: &str, ts: u64, rand_str: &str) -> String {
        let table_chars: Vec<char> = TABLE.chars().collect();

        let plaintext = format!(
            "ai={APP_ID}&di={device_id}&as={AS_VALUE}&\
             su=false&dbg=false&emu=false&hk=false&it={}&\
             ts={ts}&rt=0&os=13&dm={DEVICE_MODEL}&st={OS_TYPE}&sv={SDK_VERSION}",
            self.app_start_ts
        );

        let dyn_key = format!("v1+{rand_str}+{ts}");
        let key_enc = encode_normal_be(&dyn_key, &table_chars, I8, I9, I10);
        let big_key = make_key(&dyn_key);
        let custom_table = make_encode_table(big_key, I9, &table_chars);
        let body_enc = encode_normal_be(&plaintext, &custom_table, I8, I9, I10);

        // Token: "bEeEP" + TABLE[len(key_enc)] + key_enc + body_enc
        let separator = table_chars[key_enc.len() % table_chars.len()];
        format!("bEeEP{separator}{key_enc}{body_enc}")
    }

    /// Generate a token with auto-generated timestamp and random string.
    pub fn generate_token_auto(&self, device_id: &str) -> String {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis() as u64;

        let mut rng = rand::rng();
        let chars: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
        let rand_str: String = (0..4)
            .map(|_| chars[rng.random_range(0..chars.len())] as char)
            .collect();

        self.generate_token(device_id, ts, &rand_str)
    }
}

/// Custom encoding: NOT standard UTF-8.
/// Ported from `DynaPathMasterEngine.string2xA1s`.
///
/// For each Unicode codepoint:
/// - cp < 128: emit `cp`
/// - cp < 2048: emit `128 | ((cp >> 7) & 15)`, `cp & 127`
/// - cp >= 262144: emit `160`, `(cp >> 14) & 127`, `(cp >> 7) & 127`, `cp & 127`
/// - (63488 & cp) != 55296: emit `((cp >> 14) & 15) | 144`, `(cp >> 7) & 127`, `cp & 127`
/// - Otherwise: silently dropped (surrogates)
fn string2xa1s(data_str: &str) -> Vec<u64> {
    let mut result = Vec::new();
    for ch in data_str.chars() {
        let cp = ch as u64;
        if cp < 128 {
            result.push(cp);
        } else if cp < 2048 {
            result.push(128 | ((cp >> 7) & 15));
            result.push(cp & 127);
        } else if cp >= 262144 {
            result.push(160);
            result.push((cp >> 14) & 127);
            result.push((cp >> 7) & 127);
            result.push(cp & 127);
        } else if (63488 & cp) != 55296 {
            result.push(((cp >> 14) & 15) | 144);
            result.push((cp >> 7) & 127);
            result.push(cp & 127);
        }
        // else: silently dropped (surrogates in [2048, 262144) range where (63488 & cp) == 55296)
    }
    result
}

/// Convert key string to big integer via custom accumulation.
/// Only checks bits 0-15 (NOT bit 31).
/// Ported from `DynaPathMasterEngine.make_key`.
fn make_key(key_str: &str) -> u128 {
    let mut big_int_add: u128 = 0;
    for ch in key_str.chars() {
        let cp = ch as u128;
        let mut i9_bit: u128 = 32768; // 2^15
        for _ in 0..16 {
            if (i9_bit & cp) != 0 {
                break;
            }
            i9_bit >>= 1;
        }
        big_int_add = big_int_add.wrapping_mul(i9_bit << 1).wrapping_add(cp);
    }
    big_int_add
}

/// Factorial number system (Lehmer code) permutation.
/// Selects `encode_size` chars from `base_table` using the Lehmer decomposition of `num`.
/// Ported from `DynaPathMasterEngine.make_encode_table`.
fn make_encode_table(num: u128, encode_size: usize, base_table: &[char]) -> Vec<char> {
    let mut sb = Vec::with_capacity(encode_size);
    let mut temp_num = num;

    for i in 0..encode_size {
        let j8_divisor = (encode_size - i) as u128;
        let remainder = (temp_num % j8_divisor) as usize;
        let ch = internal_i(base_table, remainder, &sb);
        sb.push(ch);
        temp_num /= j8_divisor;
    }

    sb
}

/// Find the `remainder`-th character in `base_table` that is NOT already in `current_sb`.
fn internal_i(base_table: &[char], remainder: usize, current_sb: &[char]) -> char {
    let mut count = 0;
    for &ch in base_table {
        if !current_sb.contains(&ch) {
            if count == remainder {
                return ch;
            }
            count += 1;
        }
    }
    ' ' // fallback (should never happen with valid inputs)
}

/// Process input data through custom encoding, accumulate in base-I8,
/// decompose to I10+1 digits in base-I9, emit chars from table.
/// Ported from `DynaPathMasterEngine.encode_normal_be`.
fn encode_normal_be(data_str: &str, table: &[char], i8: u64, i9: usize, i10: usize) -> String {
    let list_data = string2xa1s(data_str);
    let mut sb = String::new();
    let mut i_arr = vec![0usize; i10 + 1];

    let size_remainder = list_data.len() % i10;
    let size2 = list_data.len() - size_remainder;
    let mut idx = 0;

    // Process full i10-byte chunks
    while idx < size2 {
        let mut val: u64 = 0;
        for _ in 0..i10 {
            val = val * i8 + list_data[idx];
            idx += 1;
        }
        for item in i_arr.iter_mut().take(i10 + 1) {
            *item = (val % i9 as u64) as usize;
            val /= i9 as u64;
        }
        for i in (0..=i10).rev() {
            if i_arr[i] < table.len() {
                sb.push(table[i_arr[i]]);
            }
        }
    }

    // Process remaining bytes
    if size_remainder > 0 {
        let mut val: u64 = 0;
        for _ in 0..size_remainder {
            val = val * i8 + list_data[idx];
            idx += 1;
        }
        for item in i_arr.iter_mut().take(size_remainder + 1) {
            *item = (val % i9 as u64) as usize;
            val /= i9 as u64;
        }
        let mut remaining = size_remainder as isize;
        while remaining >= 0 {
            if (remaining as usize) < i_arr.len() && i_arr[remaining as usize] < table.len() {
                sb.push(table[i_arr[remaining as usize]]);
            }
            remaining -= 1;
        }
    }

    sb
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn requires_token_checks() {
        assert!(requires_token("/classes/com.korail.mobile.login.Login"));
        assert!(requires_token(
            "/classes/com.korail.mobile.certification.TicketReservation"
        ));
        assert!(requires_token(
            "/classes/com.korail.mobile.seatMovie.ScheduleView"
        ));
        assert!(!requires_token("/classes/com.korail.mobile.common.code.do"));
        assert!(!requires_token(
            "/classes/com.korail.mobile.myTicket.MyTicketList"
        ));
    }

    #[test]
    fn string2xa1s_ascii() {
        let result = string2xa1s("abc");
        assert_eq!(result, vec![97, 98, 99]);
    }

    #[test]
    fn string2xa1s_mixed() {
        // "v1" should be all ASCII
        let result = string2xa1s("v1");
        assert_eq!(result, vec![118, 49]);
    }

    #[test]
    fn string2xa1s_unicode() {
        // Korean char '가' = U+AC00 = 44032
        // (63488 & 44032) = 43008, which != 55296
        // So: ((44032 >> 14) & 15) | 144 = (2 & 15) | 144 = 146
        //     (44032 >> 7) & 127 = 344 & 127 = 88
        //     44032 & 127 = 0
        let result = string2xa1s("가");
        assert_eq!(result, vec![146, 88, 0]);
    }

    #[test]
    fn make_key_simple() {
        // "v1" -> each char scanned for highest set bit in bits 0-15
        let key = make_key("v1");
        assert!(key > 0);
    }

    #[test]
    fn make_encode_table_produces_correct_length() {
        let table_chars: Vec<char> = TABLE.chars().collect();
        let result = make_encode_table(12345, I9, &table_chars);
        assert_eq!(result.len(), I9);
    }

    #[test]
    fn make_encode_table_no_duplicates() {
        let table_chars: Vec<char> = TABLE.chars().collect();
        let result = make_encode_table(99999, I9, &table_chars);
        let unique: std::collections::HashSet<char> = result.iter().copied().collect();
        assert_eq!(unique.len(), result.len());
    }

    #[test]
    fn encode_normal_be_produces_output() {
        let table_chars: Vec<char> = TABLE.chars().collect();
        let result = encode_normal_be("hello", &table_chars, I8, I9, I10);
        assert!(!result.is_empty());
    }

    #[test]
    fn encode_normal_be_deterministic() {
        let table_chars: Vec<char> = TABLE.chars().collect();
        let r1 = encode_normal_be("test data", &table_chars, I8, I9, I10);
        let r2 = encode_normal_be("test data", &table_chars, I8, I9, I10);
        assert_eq!(r1, r2);
    }

    #[test]
    fn generate_token_format() {
        let engine = DynaPathEngine::new(1710000000000);
        let token = engine.generate_token("558a4f02041657ea", 1710000001000, "AB12");
        assert!(token.starts_with("bEeEP"));
        assert!(token.len() > 10);
    }

    #[test]
    fn generate_token_deterministic() {
        let engine = DynaPathEngine::new(1710000000000);
        let t1 = engine.generate_token("558a4f02041657ea", 1710000001000, "AB12");
        let t2 = engine.generate_token("558a4f02041657ea", 1710000001000, "AB12");
        assert_eq!(t1, t2);
    }

    #[test]
    fn generate_token_varies_with_rand() {
        let engine = DynaPathEngine::new(1710000000000);
        let t1 = engine.generate_token("558a4f02041657ea", 1710000001000, "AB12");
        let t2 = engine.generate_token("558a4f02041657ea", 1710000001000, "CD34");
        assert_ne!(t1, t2);
    }

    #[test]
    fn generate_token_auto_starts_with_prefix() {
        let engine = DynaPathEngine::new(1710000000000);
        let token = engine.generate_token_auto("558a4f02041657ea");
        assert!(token.starts_with("bEeEP"));
    }
}
