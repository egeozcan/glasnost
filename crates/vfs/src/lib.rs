//! Virtual file system over the user's own game install.
//!
//! **Milestone M1** fills this crate: mounting archives (nested archives
//! included) in a configurable precedence order, case-insensitive lookup, and
//! lazy reads so the web build can read slices of user-picked files.
//!
//! Nothing here ever ships or caches game data in this repository; see
//! CONTRIBUTING.md, "No EA bytes, anywhere".
