/// The channel to hop to after `current`, following `order` and wrapping around.
///
/// If `current` is not in `order` (for example right after a pinned channel was released) hopping
/// restarts from the first entry. An empty order keeps the current channel.
pub fn next_in_order(order: &[u8], current: u8) -> u8 {
    match order.iter().position(|&c| c == current) {
        Some(i) => order[(i + 1) % order.len()],
        None => order.first().copied().unwrap_or(current),
    }
}
