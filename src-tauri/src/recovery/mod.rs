//! Crash-recovery (M1). Dirty documents are snapshotted to disk; on the next
//! launch the app offers to restore them. See DEVELOPMENT_PLAN.md §6.3.

pub mod store;

pub use store::{
    delete_all, delete_snapshot, entry_from_snapshot, read_all, write_snapshot, RecoveryEntry,
};
