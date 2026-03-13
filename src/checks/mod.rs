/// Detect forbidden token names — slint/naming.md
pub mod tokens;
/// Detect string literals in state fields — slint/globals.md
pub mod strings;
/// Detect invalid component structure — slint/structure.md
pub mod structure;
/// Detect invalid event/callback usage — slint/events.md
pub mod events;
/// Detect invalid parent-child component relationships — slint/mother-child.md
pub mod mother_child;
/// Detect string literal state comparisons — slint/types.md
pub mod string_states;
/// Detect invalid architecture layer assignments — slint/topology.md
pub mod architecture;
