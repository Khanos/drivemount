//! Run with `cargo test --release --test glib_regression` to exercise the optimized UB regression.
#![cfg(feature = "desktop")]
use glib::variant::ToVariant;
#[test]
fn variant_string_iterator_out_parameter_is_sound_when_optimized() {
    let value = vec!["zero", "one", "two", "three", "four", "five"].to_variant();
    let mut iter = value.array_iter_str().unwrap();
    assert_eq!(iter.next(), Some("zero"));
    assert_eq!(iter.nth(1), Some("two"));
    assert_eq!(iter.next_back(), Some("five"));
    assert_eq!(iter.nth_back(1), Some("three"));
    assert_eq!(value.array_iter_str().unwrap().last(), Some("five"));
}
