extern crate std;
use std::string::String;

use super::keys::parse_hex_key;

fn hex(n: usize) -> String {
    (0..n).map(|i| std::format!("{:x}", i % 16)).collect()
}

#[test]
fn parses_64_hex_characters() {
    let key = parse_hex_key(&hex(64)).unwrap();
    assert_eq!(key[0], 0x01);
    assert_eq!(key[1], 0x23);
    assert_eq!(key[31], 0xef);
}

#[test]
fn accepts_uppercase_and_surrounding_whitespace() {
    let lower = parse_hex_key(&hex(64)).unwrap();
    assert_eq!(parse_hex_key(&hex(64).to_uppercase()), Some(lower));
    assert_eq!(parse_hex_key(&std::format!("  {}\n", hex(64))), Some(lower));
}

#[test]
fn rejects_wrong_lengths() {
    for n in [0, 1, 62, 63, 65, 66, 128] {
        assert_eq!(parse_hex_key(&hex(n)), None, "length {n}");
    }
}

#[test]
fn rejects_non_hex_characters() {
    let mut bad = hex(64);
    bad.replace_range(10..11, "g");
    assert_eq!(parse_hex_key(&bad), None);
    assert_eq!(parse_hex_key(&"é".repeat(32)), None); // 64 bytes, not hex digits
    assert_eq!(parse_hex_key(&"-".repeat(64)), None);
}

#[test]
fn rejects_an_embedded_space() {
    let mut bad = hex(64);
    bad.replace_range(20..21, " ");
    assert_eq!(parse_hex_key(&bad), None);
}
