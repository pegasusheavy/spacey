// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Developer-friendly macros for the Spacey JavaScript engine.
//!
//! This crate provides ergonomic macros for common patterns in async,
//! parallel, and JavaScript engine development.
//!
//! # Module Overview
//!
//! The macros are organized into logical modules:
//!
//! - **[`collections`]** - Collection creation (hashmap, hashset, btreemap, vecdeque, etc.)
//! - **[`error`]** - Error handling (bail, ensure, try_unwrap, etc.)
//! - **[`debug`]** - Debugging and timing (time_it, debug_log, bench, etc.)
//! - **[`control`]** - Control flow (defer, retry, guard, etc.)
//! - **[`async_utils`]** - Async/parallel (spawn_blocking, join_all, par_map, etc.)
//! - **[`types`]** - Type definitions (int_enum, newtype, bitflags, etc.)
//! - **[`format`]** - Formatting (format_bytes, format_duration, progress_bar, etc.)
//! - **[`js`]** - JavaScript engine specific (js_keywords, opcodes, ast_node, etc.)
//! - **[`builder`]** - Builder patterns (config_struct, setter, appender, etc.)
//! - **[`testing`]** - Testing helpers (assert_ok, assert_matches, assert_approx_eq, etc.)
//!
//! # Quick Start
//!
//! ```rust
//! use spacey_macros::*;
//!
//! // Collection literals
//! let map = hashmap! {
//!     "key1" => "value1",
//!     "key2" => "value2",
//! };
//!
//! // Error handling
//! fn divide(a: i32, b: i32) -> Result<i32, String> {
//!     ensure!(b != 0, "division by zero");
//!     Ok(a / b)
//! }
//!
//! // Timing
//! let result = time_it!("computation", {
//!     (0..100).sum::<i32>()
//! });
//!
//! // Defer (scope guard)
//! {
//!     defer!(println!("cleanup on scope exit"));
//!     // ... do work ...
//! }
//! ```
//!
//! # Macro Categories
//!
//! ## Collections
//!
//! ```rust
//! use spacey_macros::*;
//!
//! let map = hashmap! { "a" => 1, "b" => 2 };
//! let set = hashset! { 1, 2, 3 };
//! let btree = btreemap! { "x" => 10, "y" => 20 };
//! let deque = vecdeque![1, 2, 3];
//! let vec = vec_with_capacity!(100, 1, 2, 3);
//! ```
//!
//! ## Error Handling
//!
//! ```rust
//! use spacey_macros::*;
//!
//! fn example(x: Option<i32>) -> Result<i32, String> {
//!     ensure!(x.is_some(), "x must be provided");
//!     let value = try_unwrap!(x, "x was None");
//!     if value < 0 {
//!         bail!("value must be non-negative");
//!     }
//!     Ok(value)
//! }
//! ```
//!
//! ## Type Definitions
//!
//! ```rust
//! use spacey_macros::*;
//!
//! // Integer-backed enum
//! int_enum! {
//!     #[derive(Debug, Clone, Copy, PartialEq)]
//!     pub enum Status: u8 {
//!         Active = 0,
//!         Inactive = 1,
//!     }
//! }
//!
//! // Newtype wrapper
//! newtype!(UserId, u64);
//!
//! // Bitflags
//! bitflags! {
//!     Permissions: u32 {
//!         READ = 0b001,
//!         WRITE = 0b010,
//!         EXECUTE = 0b100,
//!     }
//! }
//! ```
//!
//! ## JavaScript Engine
//!
//! ```rust
//! use spacey_macros::*;
//!
//! // Define keywords
//! js_keywords! {
//!     Keywords {
//!         "let" => Let,
//!         "const" => Const,
//!         "function" => Function,
//!     }
//! }
//!
//! // Define opcodes
//! opcodes! {
//!     Opcode: u8 {
//!         Nop = 0x00,
//!         Push = 0x01,
//!         Pop = 0x02,
//!     }
//! }
//!
//! // Check identifier characters
//! assert!(is_id_start!('_'));
//! assert!(is_id_continue!('5'));
//! ```
//!
//! ## Testing
//!
//! ```rust
//! use spacey_macros::*;
//!
//! #[test]
//! fn test_example() {
//!     let result: Result<i32, &str> = Ok(42);
//!     let value = assert_ok!(result);
//!     assert_eq!(value, 42);
//!
//!     assert_approx_eq!(0.1 + 0.2, 0.3, 1e-10);
//!     assert_contains!("hello world", "world");
//!     assert_len!(vec![1, 2, 3], 3);
//! }
//! ```

#![warn(missing_docs)]

// =============================================================================
// Module declarations
// =============================================================================

pub mod async_utils;
pub mod builder;
pub mod collections;
pub mod control;
pub mod debug;
pub mod error;
pub mod format;
pub mod js;
pub mod testing;
pub mod types;

// =============================================================================
// Re-exports for convenience
// =============================================================================

// Builder module is available but types are used internally by macros

// =============================================================================
// Helper types used by macros
// =============================================================================

/// Helper struct for defer macro.
///
/// This struct holds a closure that will be executed when it goes out of scope.
#[doc(hidden)]
pub struct DeferGuard<F: FnOnce()>(pub Option<F>);

impl<F: FnOnce()> Drop for DeferGuard<F> {
    fn drop(&mut self) {
        if let Some(f) = self.0.take() {
            f();
        }
    }
}

// =============================================================================
// Prelude - commonly used macros
// =============================================================================

/// A prelude module containing the most commonly used macros.
///
/// # Usage
///
/// ```rust
/// use spacey_macros::prelude::*;
/// ```
pub mod prelude {
    // Collections
    pub use crate::btreemap;
    pub use crate::btreeset;
    pub use crate::hashmap;
    pub use crate::hashset;
    pub use crate::linkedlist;
    pub use crate::vec_with_capacity;
    pub use crate::vecdeque;

    // Error handling
    pub use crate::bail;
    pub use crate::ensure;
    pub use crate::error_type;
    pub use crate::ok_or_break;
    pub use crate::ok_or_continue;
    pub use crate::some_or_continue;
    pub use crate::try_unwrap;
    pub use crate::try_with;

    // Debug
    pub use crate::bench;
    pub use crate::debug_log;
    pub use crate::debug_only;
    pub use crate::print_type;
    pub use crate::release_only;
    pub use crate::time_it;
    pub use crate::timed;
    pub use crate::trace_log;

    // Control flow
    pub use crate::defer;
    pub use crate::guard;
    pub use crate::if_else;
    pub use crate::loop_until;
    pub use crate::retry;
    pub use crate::scope_exit;
    pub use crate::times;
    pub use crate::times_with_index;
    pub use crate::when;

    // Types
    pub use crate::bitflags;
    pub use crate::const_assert;
    pub use crate::int_enum;
    pub use crate::newtype;
    pub use crate::str_enum;
    pub use crate::string_newtype;

    // Format
    pub use crate::clamp;
    pub use crate::format_bytes;
    pub use crate::format_duration;
    pub use crate::format_number;
    pub use crate::format_percent;
    pub use crate::indent;
    pub use crate::join;
    pub use crate::pad_left;
    pub use crate::pad_right;
    pub use crate::progress_bar;
    pub use crate::to_binary;
    pub use crate::to_hex;
    pub use crate::truncate;

    // JavaScript
    pub use crate::is_id_continue;
    pub use crate::is_id_start;
    pub use crate::is_token;
    pub use crate::js_keywords;
    pub use crate::js_object;
    pub use crate::opcodes;

    // Builder
    pub use crate::appender;
    pub use crate::config_struct;
    pub use crate::optional_setter;
    pub use crate::setter;

    // Testing
    pub use crate::assert_approx_eq;
    pub use crate::assert_contains;
    pub use crate::assert_empty;
    pub use crate::assert_err;
    pub use crate::assert_len;
    pub use crate::assert_matches;
    pub use crate::assert_none;
    pub use crate::assert_not_empty;
    pub use crate::assert_ok;
    pub use crate::assert_panics;
    pub use crate::assert_some;
    pub use crate::skip_test;

    // Helper types
    pub use crate::DeferGuard;
}

// =============================================================================
// Root-level macro re-exports (for `use spacey_macros::*;`)
// =============================================================================

// All macros are already exported at crate root via #[macro_export]

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defer_guard() {
        use std::cell::RefCell;
        let log = RefCell::new(Vec::new());

        {
            log.borrow_mut().push("start");
            let _guard = DeferGuard(Some(|| log.borrow_mut().push("end")));
            log.borrow_mut().push("middle");
        }

        assert_eq!(*log.borrow(), vec!["start", "middle", "end"]);
    }

    #[test]
    fn test_prelude_imports() {
        use crate::prelude::*;

        // Test a few macros from prelude
        let map = hashmap! { "a" => 1 };
        assert_eq!(map.get("a"), Some(&1));

        let set = hashset! { 1, 2, 3 };
        assert!(set.contains(&2));
    }
}
