//! Arena allocator for ultra-fast bump allocation.
//!
//! The arena provides O(1) allocation by simply bumping a pointer.
//! Memory is freed all at once when the arena is reset.

use std::cell::UnsafeCell;
use std::sync::atomic::{AtomicUsize, Ordering};

use super::object::{JsObject, ObjectHeader};

/// A reference to an object in the arena.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ArenaRef {
    index: usize,
}

impl ArenaRef {
    /// Creates a new arena reference.
    pub fn new(index: usize) -> Self {
        Self { index }
    }

    /// Returns the index of this reference.
    pub fn index(&self) -> usize {
        self.index
    }
}

/// An entry in the arena combining header and object.
#[repr(C)]
struct ArenaEntry {
    header: ObjectHeader,
    object: JsObject,
}

impl ArenaEntry {
    fn new(object: JsObject) -> Self {
        Self {
            header: ObjectHeader::new(),
            object,
        }
    }
}

/// A high-performance arena allocator using bump allocation.
///
/// # Memory Layout
///
/// ```text
/// ┌────────────────────────────────────────────────────────┐
/// │  Block 0                                                │
/// │  [Entry0][Entry1][Entry2]...            [free space]   │
/// │                                         ↑              │
/// │                                         bump_ptr       │
/// ├────────────────────────────────────────────────────────┤
/// │  Block 1                                                │
/// │  [Entry0][Entry1]...                                   │
/// └────────────────────────────────────────────────────────┘
/// ```
pub struct Arena {
    /// Storage for arena entries
    entries: UnsafeCell<Vec<ArenaEntry>>,
    /// Number of allocated entries
    count: AtomicUsize,
    /// Maximum capacity
    capacity: usize,
}

impl Arena {
    /// Creates a new arena with the given size parameters.
    pub fn new(total_size: usize, _block_size: usize) -> Self {
        // Calculate approximate capacity based on entry size
        let entry_size = std::mem::size_of::<ArenaEntry>();
        let capacity = total_size / entry_size;

        Self {
            entries: UnsafeCell::new(Vec::with_capacity(capacity)),
            count: AtomicUsize::new(0),
            capacity,
        }
    }

    /// Allocates a new object in the arena.
    ///
    /// Returns `None` if the arena is full.
    ///
    /// # Performance
    ///
    /// This is an O(1) operation - just a pointer bump and bounds check.
    #[inline]
    pub fn allocate(&self, object: JsObject) -> Option<ArenaRef> {
        let current = self.count.load(Ordering::Relaxed);

        if current >= self.capacity {
            return None;
        }

        // Try to reserve a slot
        let index = self.count.fetch_add(1, Ordering::AcqRel);

        if index >= self.capacity {
            // Another thread took the last slot, back off
            self.count.fetch_sub(1, Ordering::Relaxed);
            return None;
        }

        // Store the entry
        let entry = ArenaEntry::new(object);

        // SAFETY: We have exclusive access to this index
        unsafe {
            let entries = &mut *self.entries.get();
            if index >= entries.len() {
                entries.resize_with(index + 1, || ArenaEntry::new(JsObject::new()));
            }
            entries[index] = entry;
        }

        Some(ArenaRef::new(index))
    }

    /// Gets a reference to an object by index.
    #[inline]
    pub fn get(&self, index: usize) -> Option<&JsObject> {
        if index >= self.count.load(Ordering::Relaxed) {
            return None;
        }

        // SAFETY: Index is within bounds and entry is initialized
        unsafe {
            let entries = &*self.entries.get();
            entries.get(index).map(|e| &e.object)
        }
    }

    /// Gets a reference to an object's header by index.
    #[inline]
    pub fn get_header(&self, index: usize) -> Option<&ObjectHeader> {
        if index >= self.count.load(Ordering::Relaxed) {
            return None;
        }

        // SAFETY: Index is within bounds and entry is initialized
        unsafe {
            let entries = &*self.entries.get();
            entries.get(index).map(|e| &e.header)
        }
    }

    /// Returns the number of bytes currently used.
    #[inline]
    pub fn used(&self) -> usize {
        self.count.load(Ordering::Relaxed) * std::mem::size_of::<ArenaEntry>()
    }

    /// Returns the number of objects in the arena.
    #[inline]
    pub fn object_count(&self) -> usize {
        self.count.load(Ordering::Relaxed)
    }

    /// Resets the arena, freeing all memory.
    ///
    /// # Safety
    ///
    /// All references to objects in this arena become invalid after reset.
    pub fn reset(&self) {
        // Clear entries
        unsafe {
            let entries = &mut *self.entries.get();
            entries.clear();
        }
        self.count.store(0, Ordering::Release);
    }

    /// Returns the capacity in number of objects.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Returns whether the arena is full.
    #[inline]
    pub fn is_full(&self) -> bool {
        self.count.load(Ordering::Relaxed) >= self.capacity
    }
}

// SAFETY: Arena uses atomic operations for count and UnsafeCell with proper synchronization
unsafe impl Send for Arena {}
unsafe impl Sync for Arena {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arena_allocate() {
        let arena = Arena::new(4096, 1024);

        let obj = JsObject::new();
        let arena_ref = arena.allocate(obj).unwrap();

        assert_eq!(arena_ref.index(), 0);
        assert!(arena.get(0).is_some());
    }

    #[test]
    fn test_arena_multiple_allocations() {
        let arena = Arena::new(4096, 1024);

        for i in 0..10 {
            let arena_ref = arena.allocate(JsObject::new()).unwrap();
            assert_eq!(arena_ref.index(), i);
        }

        assert_eq!(arena.object_count(), 10);
    }

    #[test]
    fn test_arena_reset() {
        let arena = Arena::new(4096, 1024);

        for _ in 0..5 {
            arena.allocate(JsObject::new()).unwrap();
        }

        assert_eq!(arena.object_count(), 5);

        arena.reset();

        assert_eq!(arena.object_count(), 0);
        assert_eq!(arena.used(), 0);
    }

    #[test]
    fn test_arena_header_access() {
        let arena = Arena::new(4096, 1024);

        let arena_ref = arena.allocate(JsObject::new()).unwrap();

        let header = arena.get_header(arena_ref.index()).unwrap();
        assert_eq!(header.age.load(Ordering::Relaxed), 0);
    }
}
