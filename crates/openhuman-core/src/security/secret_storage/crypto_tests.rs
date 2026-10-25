use super::*;

#[test]
fn hex_decode_rejects_non_ascii_input_without_panicking() {
    // Even byte length, but byte 2 falls inside 'é'.
    let decoded = std::panic::catch_unwind(|| hex_decode("aéb"))
        .expect("hex_decode panicked on non-ASCII input");
    assert!(decoded.is_err());
}
