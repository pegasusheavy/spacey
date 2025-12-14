//! Developer-friendly macros for the Spacey JavaScript engine.
//!
//! This crate provides ergonomic macros for common patterns in async,
//! parallel, and JavaScript engine development.
//!
//! # Macros Overview
//!
//! ## Async/Parallel
//! - [`parallel_threshold!`] - Conditionally parallelize based on collection size
//! - [`spawn_blocking!`] - Spawn blocking work with proper error handling
//! - [`join_all!`] - Join multiple async operations
//!
//! ## Error Handling
//! - [`bail!`] - Early return with an error
//! - [`ensure!`] - Assertion that returns an error instead of panicking
//!
//! ## Collections
//! - [`hashmap!`] - Create a HashMap literal
//! - [`hashset!`] - Create a HashSet literal
//!
//! ## Development
//! - [`time_it!`] - Measure execution time of a block
//! - [`debug_log!`] - Debug logging that compiles out in release
//! - [`defer!`] - Execute code on scope exit (like Go's defer)
//! - [`const_assert!`] - Compile-time assertions
//!
//! ## Utilities
//! - [`try_unwrap!`] - Unwrap Option or return error
//! - [`vec_with_capacity!`] - Create Vec with preallocated capacity
//! - [`int_enum!`] - Create enums convertible to/from integers
//! - [`retry!`] - Retry with exponential backoff
//!
//! # Examples
//!
//! ```
//! use spacey_macros::*;
//!
//! // Collection literals
//! let map = hashmap! {
//!     "key1" => "value1",
//!     "key2" => "value2",
//! };
//! assert_eq!(map.get("key1"), Some(&"value1"));
//!
//! // HashSet literal
//! let set = hashset! { 1, 2, 3 };
//! assert!(set.contains(&2));
//! ```
//!
//! ## Error Handling
//!
//! ```
//! use spacey_macros::*;
//!
//! fn divide(a: i32, b: i32) -> Result<i32, String> {
//!     ensure!(b != 0, "division by zero");
//!     Ok(a / b)
//! }
//!
//! assert_eq!(divide(10, 2).unwrap(), 5);
//! assert!(divide(10, 0).is_err());
//! ```
//!
//! ## Timing
//!
//! ```
//! use spacey_macros::*;
//!
//! let result = time_it!("computation", {
//!     (0..100).sum::<i32>()
//! });
//! assert_eq!(result, 4950);
//! ```
//!
//! ## Defer (scope guard)
//!
//! ```
//! use spacey_macros::*;
//! use std::cell::RefCell;
//!
//! let log = RefCell::new(Vec::new());
//!
//! {
//!     log.borrow_mut().push("start");
//!     defer!(log.borrow_mut().push("cleanup"));
//!     log.borrow_mut().push("work");
//! }
//!
//! assert_eq!(*log.borrow(), vec!["start", "work", "cleanup"]);
//! ```

#![warn(missing_docs)]

/// Creates a HashMap from key-value pairs.
///
/// # Example
///
/// ```
/// use spacey_macros::hashmap;
///
/// let map = hashmap! {
///     "key1" => "value1",
///     "key2" => "value2",
/// };
/// assert_eq!(map.get("key1"), Some(&"value1"));
/// ```
#[macro_export]
macro_rules! hashmap {
    () => {
        ::std::collections::HashMap::new()
    };
    ($($key:expr => $value:expr),+ $(,)?) => {{
        let mut map = ::std::collections::HashMap::new();
        $(map.insert($key, $value);)+
        map
    }};
}

/// Creates a HashSet from values.
///
/// # Example
///
/// ```
/// use spacey_macros::hashset;
///
/// let set = hashset! { 1, 2, 3 };
/// assert!(set.contains(&2));
/// ```
#[macro_export]
macro_rules! hashset {
    () => {
        ::std::collections::HashSet::new()
    };
    ($($value:expr),+ $(,)?) => {{
        let mut set = ::std::collections::HashSet::new();
        $(set.insert($value);)+
        set
    }};
}

/// Early return with an error.
///
/// # Example
///
/// ```
/// use spacey_macros::bail;
///
/// fn process(x: i32) -> Result<(), String> {
///     if x < 0 {
///         bail!("x must be non-negative, got {}", x);
///     }
///     Ok(())
/// }
///
/// assert!(process(-1).is_err());
/// assert!(process(1).is_ok());
/// ```
#[macro_export]
macro_rules! bail {
    ($msg:literal $(,)?) => {
        return Err($msg.into())
    };
    ($fmt:literal, $($arg:tt)*) => {
        return Err(format!($fmt, $($arg)*).into())
    };
    ($err:expr $(,)?) => {
        return Err($err.into())
    };
}

/// Ensure a condition is true, or return an error.
///
/// # Example
///
/// ```
/// use spacey_macros::ensure;
///
/// fn divide(a: i32, b: i32) -> Result<i32, String> {
///     ensure!(b != 0, "division by zero");
///     Ok(a / b)
/// }
///
/// assert!(divide(10, 0).is_err());
/// assert_eq!(divide(10, 2).unwrap(), 5);
/// ```
#[macro_export]
macro_rules! ensure {
    ($cond:expr, $msg:literal $(,)?) => {
        if !$cond {
            return Err($msg.into());
        }
    };
    ($cond:expr, $fmt:literal, $($arg:tt)*) => {
        if !$cond {
            return Err(format!($fmt, $($arg)*).into());
        }
    };
}

/// Measure and print execution time of a block.
///
/// # Example
///
/// ```
/// use spacey_macros::time_it;
///
/// let result = time_it!("computation", {
///     let mut sum = 0;
///     for i in 0..100 {
///         sum += i;
///     }
///     sum
/// });
/// assert_eq!(result, 4950);
/// ```
#[macro_export]
macro_rules! time_it {
    ($name:expr, $block:expr) => {{
        let __start = ::std::time::Instant::now();
        let __result = $block;
        let __elapsed = __start.elapsed();
        eprintln!("[TIMER] {} took {:?}", $name, __elapsed);
        __result
    }};
}

/// Debug log that compiles out in release builds.
///
/// # Example
///
/// ```
/// use spacey_macros::debug_log;
///
/// debug_log!("processing item: {}", 42);
/// ```
#[macro_export]
macro_rules! debug_log {
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)]
        eprintln!("[DEBUG] {}", format!($($arg)*));
    };
}

/// Conditionally use parallel iteration based on threshold.
///
/// Requires `rayon` crate.
///
/// # Example
///
/// ```ignore
/// use spacey_macros::parallel_threshold;
///
/// let items: Vec<i32> = (0..10000).collect();
/// let results = parallel_threshold!(items, 1000,
///     |item| item * 2
/// );
/// ```
#[macro_export]
macro_rules! parallel_threshold {
    ($collection:expr, $threshold:expr, |$item:ident| $transform:expr) => {{
        if $collection.len() >= $threshold {
            use ::rayon::prelude::*;
            $collection
                .par_iter()
                .map(|$item| $transform)
                .collect::<Vec<_>>()
        } else {
            $collection
                .iter()
                .map(|$item| $transform)
                .collect::<Vec<_>>()
        }
    }};
}

/// Spawn a blocking task for CPU-intensive work.
///
/// Requires `tokio` crate.
///
/// # Example
///
/// ```ignore
/// use spacey_macros::spawn_blocking;
///
/// async fn example() {
///     let result = spawn_blocking!(expensive_computation()).await.unwrap();
/// }
/// ```
#[macro_export]
macro_rules! spawn_blocking {
    ($block:expr) => {
        ::tokio::task::spawn_blocking(move || $block)
    };
}

/// Join multiple async operations concurrently.
///
/// Requires `tokio` crate.
///
/// # Example
///
/// ```ignore
/// use spacey_macros::join_all;
///
/// async fn example() {
///     let (a, b, c) = join_all!(
///         fetch_a(),
///         fetch_b(),
///         fetch_c(),
///     );
/// }
/// ```
#[macro_export]
macro_rules! join_all {
    ($($future:expr),+ $(,)?) => {
        ::tokio::join!($($future),+)
    };
}

/// Create a Vec with pre-allocated capacity.
///
/// # Example
///
/// ```
/// use spacey_macros::vec_with_capacity;
///
/// let items = vec_with_capacity!(100, 1, 2, 3);
/// assert!(items.capacity() >= 100);
/// assert_eq!(items.len(), 3);
/// ```
#[macro_export]
macro_rules! vec_with_capacity {
    ($capacity:expr) => {
        Vec::with_capacity($capacity)
    };
    ($capacity:expr, $($item:expr),+ $(,)?) => {{
        let mut v = Vec::with_capacity($capacity);
        $(v.push($item);)+
        v
    }};
}

/// Unwrap an Option or return early with an error.
///
/// # Example
///
/// ```
/// use spacey_macros::try_unwrap;
///
/// fn get_value(opt: Option<i32>) -> Result<i32, String> {
///     let value = try_unwrap!(opt, "value was None");
///     Ok(value * 2)
/// }
///
/// assert_eq!(get_value(Some(5)).unwrap(), 10);
/// assert!(get_value(None).is_err());
/// ```
#[macro_export]
macro_rules! try_unwrap {
    ($opt:expr, $msg:literal) => {
        match $opt {
            Some(v) => v,
            None => return Err($msg.into()),
        }
    };
    ($opt:expr) => {
        match $opt {
            Some(v) => v,
            None => return Err("unwrap failed on None".into()),
        }
    };
}

/// Assert at compile time.
///
/// # Example
///
/// ```
/// use spacey_macros::const_assert;
///
/// const_assert!(std::mem::size_of::<u64>() == 8);
/// ```
#[macro_export]
macro_rules! const_assert {
    ($cond:expr) => {
        const _: () = assert!($cond);
    };
    ($cond:expr, $msg:literal) => {
        const _: () = assert!($cond, $msg);
    };
}

/// Defer execution until scope exit (like Go's defer).
///
/// # Example
///
/// ```
/// use spacey_macros::defer;
///
/// fn example() {
///     defer!(println!("cleanup"));
///     println!("work");
///     // prints: work, then cleanup
/// }
/// ```
#[macro_export]
macro_rules! defer {
    ($($body:tt)*) => {
        let _guard = $crate::DeferGuard(Some(|| { $($body)* }));
    };
}

/// Helper struct for defer macro.
#[doc(hidden)]
pub struct DeferGuard<F: FnOnce()>(pub Option<F>);

impl<F: FnOnce()> Drop for DeferGuard<F> {
    fn drop(&mut self) {
        if let Some(f) = self.0.take() {
            f();
        }
    }
}

/// Retry an operation with exponential backoff.
///
/// # Example
///
/// ```
/// use spacey_macros::retry;
///
/// fn fallible() -> Result<i32, &'static str> {
///     static mut ATTEMPTS: i32 = 0;
///     unsafe {
///         ATTEMPTS += 1;
///         if ATTEMPTS < 3 {
///             Err("not yet")
///         } else {
///             Ok(42)
///         }
///     }
/// }
///
/// let result = retry!(5, fallible());
/// assert!(result.is_ok());
/// ```
#[macro_export]
macro_rules! retry {
    ($max_attempts:expr, $op:expr) => {{
        let mut attempts = 0;
        loop {
            attempts += 1;
            match $op {
                Ok(v) => break Ok(v),
                Err(e) if attempts >= $max_attempts => break Err(e),
                Err(_) => {
                    ::std::thread::sleep(::std::time::Duration::from_millis(
                        10 * (1 << attempts.min(6)),
                    ));
                }
            }
        }
    }};
}

/// Create an enum that can convert to/from integers.
///
/// # Example
///
/// ```
/// use spacey_macros::int_enum;
///
/// int_enum! {
///     #[derive(Debug, Clone, Copy, PartialEq)]
///     pub enum Color: u8 {
///         Red = 0,
///         Green = 1,
///         Blue = 2,
///     }
/// }
///
/// assert_eq!(Color::Green as u8, 1);
/// assert_eq!(Color::try_from(2u8).unwrap(), Color::Blue);
/// ```
#[macro_export]
macro_rules! int_enum {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident : $repr:ty {
            $($variant:ident = $value:expr),+ $(,)?
        }
    ) => {
        $(#[$meta])*
        #[repr($repr)]
        $vis enum $name {
            $($variant = $value),+
        }

        impl TryFrom<$repr> for $name {
            type Error = ();

            fn try_from(value: $repr) -> Result<Self, ()> {
                match value {
                    $($value => Ok(Self::$variant),)+
                    _ => Err(()),
                }
            }
        }
    };
}

/// Measure execution time and return both duration and result.
///
/// # Example
///
/// ```
/// use spacey_macros::timed;
///
/// let (duration, result) = timed!({
///     (0..100).sum::<i32>()
/// });
///
/// assert_eq!(result, 4950);
/// assert!(duration.as_nanos() > 0);
/// ```
#[macro_export]
macro_rules! timed {
    ($block:expr) => {{
        let __start = ::std::time::Instant::now();
        let __result = $block;
        let __elapsed = __start.elapsed();
        (__elapsed, __result)
    }};
}

/// Format bytes as human-readable size.
///
/// # Example
///
/// ```
/// use spacey_macros::format_bytes;
///
/// assert_eq!(format_bytes!(1024), "1.00 KB");
/// assert_eq!(format_bytes!(1024 * 1024), "1.00 MB");
/// assert_eq!(format_bytes!(500), "500 B");
/// ```
#[macro_export]
macro_rules! format_bytes {
    ($bytes:expr) => {{
        let bytes = $bytes as f64;
        if bytes >= 1_073_741_824.0 {
            format!("{:.2} GB", bytes / 1_073_741_824.0)
        } else if bytes >= 1_048_576.0 {
            format!("{:.2} MB", bytes / 1_048_576.0)
        } else if bytes >= 1024.0 {
            format!("{:.2} KB", bytes / 1024.0)
        } else {
            format!("{} B", bytes as u64)
        }
    }};
}

/// Clamp a value between min and max.
///
/// # Example
///
/// ```
/// use spacey_macros::clamp;
///
/// assert_eq!(clamp!(5, 0, 10), 5);
/// assert_eq!(clamp!(-5, 0, 10), 0);
/// assert_eq!(clamp!(15, 0, 10), 10);
/// ```
#[macro_export]
macro_rules! clamp {
    ($value:expr, $min:expr, $max:expr) => {{
        let v = $value;
        let min = $min;
        let max = $max;
        if v < min {
            min
        } else if v > max {
            max
        } else {
            v
        }
    }};
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_hashmap() {
        let map = hashmap! {
            "a" => 1,
            "b" => 2,
        };
        assert_eq!(map.get("a"), Some(&1));
        assert_eq!(map.get("b"), Some(&2));
    }

    #[test]
    fn test_hashset() {
        let set = hashset! { 1, 2, 3 };
        assert!(set.contains(&1));
        assert!(set.contains(&2));
        assert!(set.contains(&3));
        assert!(!set.contains(&4));
    }

    #[test]
    fn test_ensure() {
        fn check(x: i32) -> Result<(), String> {
            ensure!(x > 0, "must be positive");
            Ok(())
        }

        assert!(check(1).is_ok());
        assert!(check(0).is_err());
    }

    #[test]
    fn test_bail() {
        fn early_return(fail: bool) -> Result<i32, String> {
            if fail {
                bail!("failed");
            }
            Ok(42)
        }

        assert_eq!(early_return(false).unwrap(), 42);
        assert!(early_return(true).is_err());
    }

    #[test]
    fn test_vec_with_capacity() {
        let v = vec_with_capacity!(100, 1, 2, 3);
        assert!(v.capacity() >= 100);
        assert_eq!(v, vec![1, 2, 3]);
    }

    #[test]
    fn test_try_unwrap() {
        fn get(opt: Option<i32>) -> Result<i32, String> {
            let v = try_unwrap!(opt, "none");
            Ok(v)
        }

        assert_eq!(get(Some(5)).unwrap(), 5);
        assert!(get(None).is_err());
    }

    #[test]
    fn test_const_assert() {
        const_assert!(1 + 1 == 2);
        const_assert!(std::mem::size_of::<u8>() == 1);
    }

    #[test]
    fn test_defer() {
        use std::cell::RefCell;
        let order = RefCell::new(Vec::new());

        {
            order.borrow_mut().push(1);
            defer!(order.borrow_mut().push(3));
            order.borrow_mut().push(2);
        }

        assert_eq!(*order.borrow(), vec![1, 2, 3]);
    }

    #[test]
    fn test_int_enum() {
        int_enum! {
            #[derive(Debug, Clone, Copy, PartialEq)]
            enum Status: u8 {
                Success = 0,
                Failed = 1,
                Pending = 2,
            }
        }

        assert_eq!(Status::Success as u8, 0);
        assert_eq!(Status::try_from(1u8).unwrap(), Status::Failed);
        assert_eq!(Status::try_from(2u8).unwrap(), Status::Pending);
        assert!(Status::try_from(99u8).is_err());
    }

    #[test]
    fn test_timed() {
        let (duration, result) = timed!({
            let mut sum = 0;
            for i in 0..100 {
                sum += i;
            }
            sum
        });

        assert_eq!(result, 4950);
        assert!(duration.as_nanos() > 0);
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes!(512), "512 B");
        assert_eq!(format_bytes!(1024), "1.00 KB");
        assert_eq!(format_bytes!(1536), "1.50 KB");
        assert_eq!(format_bytes!(1048576), "1.00 MB");
        assert_eq!(format_bytes!(1073741824u64), "1.00 GB");
    }

    #[test]
    fn test_clamp() {
        assert_eq!(clamp!(5, 0, 10), 5);
        assert_eq!(clamp!(-5, 0, 10), 0);
        assert_eq!(clamp!(15, 0, 10), 10);
        assert_eq!(clamp!(0.5, 0.0, 1.0), 0.5);
    }
}
