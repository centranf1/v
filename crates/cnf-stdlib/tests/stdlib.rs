//! Unit tests for cnf-stdlib

use cnf_stdlib::*;

#[test]
fn test_string_basic() {
    assert!(is_empty(""));
    assert_eq!(length("abc"), 3);
    assert_eq!(to_upper("abc"), "ABC");
    assert_eq!(to_lower("ABC"), "abc");
    assert_eq!(trim("  x "), "x");
    assert_eq!(substring("abcdef", 2, 3), "cde");
    assert_eq!(pad_left("x", 3, '0'), "00x");
    assert_eq!(pad_right("x", 3, '0'), "x00");
    assert_eq!(split("a,b,c", ','), vec!["a", "b", "c"]);
    assert_eq!(parse_int("42").unwrap(), 42);
    assert_eq!(format_template("{0}-{1}", &["A", "B"]), "A-B");
    assert_eq!(reverse("abc"), "cba");
    assert_eq!(find("abcdef", "cd"), Some(2));
    assert_eq!(replace("abcabc", "a", "z"), "zbczbc");
    assert_eq!(remove_whitespace("a b\tc\n"), "abc");
    assert_eq!(capitalize("hello"), "Hello");
    assert_eq!(join(&["a", "b"], ","), "a,b");
}

#[test]
fn test_buffer_basic() {
    let b = b"abc";
    assert_eq!(size(b), 3);
    assert!(!is_empty(b));
    assert_eq!(hex_encode(b), "616263");
    assert_eq!(hex_decode("616263").unwrap(), b);
    assert_eq!(concat(b, b"d"), b"abcd");
    assert_eq!(slice(b, 1, 2), b"bc");
    assert_eq!(xor(b, b).unwrap(), vec![0,0,0]);
    assert!(find_pattern(b"abcdef", b"cd").is_some());
}

#[test]
fn test_math_basic() {
    assert_eq!(max(2, 3), 3);
    assert_eq!(min(2, 3), 2);
    assert_eq!(abs(-5), 5);
    assert_eq!(checked_add(1, 2), Some(3));
    assert_eq!(checked_sub(5, 2), Some(3));
    assert_eq!(checked_mul(2, 3), Some(6));
    assert_eq!(checked_div(6, 2), Some(3));
    assert_eq!(round_to(3.14159, 2), 3.14);
}

#[test]
fn test_collection_basic() {
    let v = vec![1,2,2,3];
    assert_eq!(count(&v), 4);
    assert_eq!(find(&v, &2), Some(1));
    assert_eq!(unique(&v), vec![1,2,3]);
    assert_eq!(sum(&[1,2,3]), 6);
    assert!(mean(&[1.0,2.0,3.0]).unwrap() - 2.0 < 1e-6);
}

#[test]
fn test_io_basic() {
    assert_eq!(display("x"), "[DISPLAY] x");
    assert_eq!(print("y"), "[PRINT] y");
    assert_eq!(read("z"), "z");
}

#[test]
fn test_convert_basic() {
    let i = 42i64;
    let b = i64_to_buffer(i);
    assert_eq!(buffer_to_i64(&b), Some(i));
    let f = 3.14f64;
    let bf = f64_to_buffer(f);
    assert_eq!(buffer_to_f64(&bf), Some(f));
    let s = "abc";
    let bs = str_to_buffer(s);
    assert_eq!(buffer_to_str(&bs).unwrap(), s);
}

#[test]
fn test_compress_basic() {
    let d = b"abc";
    let r = compress_v153(d);
    assert_eq!(r.compressed, d);
    assert!(entropy_heuristic(d) > 0.0);
}

#[test]
fn test_integrity_basic() {
    let d = b"abc";
    let h = sha256_digest(d);
    assert!(verify(d, &h));
    let hex = sha256_hex(d);
    assert!(verify_strict(d, &hex));
    let p = digest_pipeline(&[b"a", b"b"]);
    assert_ne!(h, p);
}

#[test]
fn test_crypto_basic() {
    let key = [0u8; 32];
    let nonce = [0u8; 12];
    let pt = b"abc";
    let ct = aes256gcm_encrypt(&key, &nonce, pt).unwrap();
    let dec = aes256gcm_decrypt(&key, &nonce, &ct).unwrap();
    assert_eq!(pt, &dec[..]);
    assert!(round_trip_verify(&key, &nonce, pt));
}

#[test]
fn test_format_basic() {
    assert_eq!(detect_format(b"{\"a\":1}"), "JSON");
    assert!(validate_json("{\"a\":1}"));
    assert!(validate_csv("a,b\nc,d"));
    assert!(validate_xml("<a>1</a>"));
}

#[test]
fn test_time_basic() {
    let ts = now_timestamp();
    let iso = format_iso8601(ts);
    assert!(iso.contains('T'));
    let sw = Stopwatch::start();
    assert!(sw.elapsed() >= 0);
}

#[test]
fn test_env_basic() {
    assert!(!os().is_empty());
    assert!(!arch().is_empty());
    assert!(cpu_count() > 0);
    let _info = RuntimeInfo::gather();
}
