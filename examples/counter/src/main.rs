//! Minimal executable that exercises the handwritten CXX bridge scaffold.

/// Handwritten CXX bridge declarations for the counter example.
pub mod bridge;

fn main() {
    assert_eq!(bridge::count_width(), 8);
    assert!(bridge::reset_is_active_low());

    println!("counter example scaffold ready");
}
