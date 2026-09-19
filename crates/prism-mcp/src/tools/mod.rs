//! Tool handler modules for PrismServer (BC-2.10.002).
//!
//! All tool handlers are implemented as methods on `PrismServer` in `crate::server`.
//! Registration uses two `#[tool_router]` blocks — `#[tool_router(router = live_tool_router)]`
//! for always-available tools and `#[tool_router(router = operations_tool_router)]` for the
//! `operations` feature — combined by a manual `fn tool_router()` function (D-1110 pattern).
//! The operations block is compiled only when `--features operations` is passed.
//!
//! Tool categories per BC-2.13.* catalog:
//! - `query`          — PrismQL query execution + explain + alias CRUD (PrismServer methods)
//! - `write`          — confirm_action (irreversible write confirmation, PrismServer method)
//! - `sensor_health`  — sensor connectivity + diagnostics (PrismServer methods)
//! - `config`         — config reload + sensor spec management + capability listing (PrismServer methods)
//! - `operations`     — schedule / detection / case management; gated behind the
//!   default-OFF `operations` Cargo feature. ABSENT (not registered, not in
//!   tools/list) in the default build; when built with `--features operations`
//!   the handlers compile, register, and return error code -32003
//!   (INV-OPERATIONS-FEATURE-GATE, BC-2.10.017)
//!
//! # Injection Defense (BC-2.09.001 — NON-NEGOTIABLE)
//!
//! Every tool handler method on PrismServer calls `scan_inputs(self.injection_scanner, ...)`
//! before any domain logic. There are no exempt tool paths. This is enforced structurally
//! via the `injection_scanner` field on PrismServer (constructed at boot via `new()`).

pub mod config;
#[cfg(feature = "operations")]
pub mod operations;
/// `prism_describe` L2 schema discovery tool (BC-2.10.012).
///
/// Implements the `prism_describe` MCP tool: returns the PQL-queryable table
/// and column schema for a given client (org), optionally filtered to a
/// specific sensor. Response type is `PrismDescribeOutput` (JSON) carrying
/// `DescribeTable` entries with column names, types, and descriptions.
pub mod prism_describe;
pub mod query;
pub mod sensor_health;
pub mod write;
