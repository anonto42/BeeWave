/// Where an update is. An `enum` can carry different data per variant,
/// so impossible states (e.g. "receiving" with no size) cannot be represented.
#[derive(Debug, PartialEq, Eq)]
pub(super) enum State {
    Idle,
    Receiving { size: u32, sha: [u8; 32], next: u32 },
}
