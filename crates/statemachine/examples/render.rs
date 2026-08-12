//! Regeneration aid, not a gate: prints `published_markdown()` so a
//! spec change can rewrite `schemas/state-machine.md` in one step —
//!
//! ```text
//! cargo run -p partman-statemachine --example render > schemas/state-machine.md
//! ```
//!
//! The `the_published_table_is_byte_fresh` test arbitrates the result;
//! this example performs no repository write itself.

fn main() {
    print!("{}", partman_statemachine::published_markdown());
}
