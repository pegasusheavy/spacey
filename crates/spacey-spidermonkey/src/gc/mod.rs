//! Garbage collector for the JavaScript runtime.
//!
//! This module implements a mark-and-sweep garbage collector with optional
//! parallel marking using rayon.
//!
//! ## Architecture
//!
//! The GC consists of:
//! - A heap of `GcObject` instances
//! - A set of root references
//! - Mark-and-sweep collection algorithm
//! - Optional parallel marking for large heaps
//!
//! ## Usage
//!
//! ```ignore
//! let mut heap = Heap::new();
//! let obj_ref = heap.allocate(JsObject::new());
//! // ... use obj_ref ...
//! heap.collect(); // Run garbage collection
//! ```
//!
//! ## Parallel Collection
//!
//! When the `parallel` feature is enabled and the heap is large enough,
//! the mark phase runs in parallel using rayon:
//!
//! ```ignore
//! heap.set_parallel_threshold(1000); // Enable parallel marking for heaps > 1000 objects
//! heap.collect(); // Uses parallel marking if threshold is met
//! ```

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};

#[cfg(feature = "parallel")]
use rayon::prelude::*;

use parking_lot::RwLock;

/// A reference to a garbage-collected object.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GcRef(usize);

impl GcRef {
    /// Returns the raw index of this reference.
    pub fn index(&self) -> usize {
        self.0
    }
}

/// Trait for types that can be garbage collected.
pub trait GcTrace: Send + Sync {
    /// Returns all GC references held by this object.
    fn trace_refs(&self) -> Vec<GcRef>;
}

/// A garbage-collected heap object wrapper.
///
/// Uses atomic marking for thread-safe parallel collection.
struct GcCell<T> {
    /// The object data
    value: RwLock<T>,
    /// Whether this object has been marked during GC (atomic for parallel marking)
    marked: AtomicBool,
}

/// A JavaScript object stored on the heap.
#[derive(Debug, Clone)]
pub struct JsObject {
    /// The prototype reference (if any)
    pub prototype: Option<GcRef>,
    /// Object properties
    pub properties: HashMap<String, PropertyValue>,
    /// Whether the object is extensible
    pub extensible: bool,
    /// Whether the object is sealed
    pub sealed: bool,
    /// Whether the object is frozen
    pub frozen: bool,
}

impl JsObject {
    /// Creates a new empty object.
    pub fn new() -> Self {
        Self {
            prototype: None,
            properties: HashMap::new(),
            extensible: true,
            sealed: false,
            frozen: false,
        }
    }

    /// Creates a new object with a prototype.
    pub fn with_prototype(prototype: GcRef) -> Self {
        Self {
            prototype: Some(prototype),
            properties: HashMap::new(),
            extensible: true,
            sealed: false,
            frozen: false,
        }
    }

    /// Gets a property value.
    pub fn get(&self, name: &str) -> Option<&PropertyValue> {
        self.properties.get(name)
    }

    /// Sets a property value.
    pub fn set(&mut self, name: String, value: PropertyValue) {
        if !self.frozen {
            self.properties.insert(name, value);
        }
    }

    /// Deletes a property.
    pub fn delete(&mut self, name: &str) -> bool {
        if self.sealed || self.frozen {
            false
        } else {
            self.properties.remove(name).is_some()
        }
    }
}

impl Default for JsObject {
    fn default() -> Self {
        Self::new()
    }
}

impl GcTrace for JsObject {
    fn trace_refs(&self) -> Vec<GcRef> {
        let mut refs = Vec::new();
        if let Some(proto) = self.prototype {
            refs.push(proto);
        }
        for value in self.properties.values() {
            if let PropertyValue::Object(obj_ref) = value {
                refs.push(*obj_ref);
            }
        }
        refs
    }
}

/// A property value that can reference other objects.
#[derive(Debug, Clone)]
pub enum PropertyValue {
    /// undefined
    Undefined,
    /// null
    Null,
    /// Boolean value
    Boolean(bool),
    /// Number value
    Number(f64),
    /// String value
    String(String),
    /// Object reference
    Object(GcRef),
}

/// The garbage-collected heap.
pub struct Heap {
    /// All allocated objects
    objects: Vec<Option<GcCell<JsObject>>>,
    /// Free list for reusing slots
    free_list: Vec<usize>,
    /// Root references
    roots: Vec<GcRef>,
    /// Number of allocations since last GC
    allocations_since_gc: usize,
    /// Threshold for triggering GC
    gc_threshold: usize,
    /// Threshold for enabling parallel marking
    parallel_threshold: usize,
    /// Statistics
    stats: GcStats,
}

/// GC statistics.
#[derive(Debug, Clone, Default)]
pub struct GcStats {
    /// Total number of collections
    pub collections: usize,
    /// Total objects allocated
    pub total_allocated: usize,
    /// Total objects freed
    pub total_freed: usize,
    /// Current live objects
    pub live_objects: usize,
}

impl Heap {
    /// Creates a new empty heap.
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
            free_list: Vec::new(),
            roots: Vec::new(),
            allocations_since_gc: 0,
            gc_threshold: 100,
            parallel_threshold: 1000,
            stats: GcStats::default(),
        }
    }

    /// Allocates a new object on the heap.
    pub fn allocate(&mut self, object: JsObject) -> GcRef {
        self.allocations_since_gc += 1;
        self.stats.total_allocated += 1;
        self.stats.live_objects += 1;

        // Check if we should run GC
        if self.allocations_since_gc >= self.gc_threshold {
            self.collect();
        }

        let cell = GcCell {
            value: RwLock::new(object),
            marked: AtomicBool::new(false),
        };

        let index = if let Some(free_idx) = self.free_list.pop() {
            self.objects[free_idx] = Some(cell);
            free_idx
        } else {
            let idx = self.objects.len();
            self.objects.push(Some(cell));
            idx
        };

        GcRef(index)
    }

    /// Gets a read guard to an object.
    pub fn get(&self, gc_ref: GcRef) -> Option<parking_lot::RwLockReadGuard<'_, JsObject>> {
        self.objects
            .get(gc_ref.0)
            .and_then(|opt| opt.as_ref())
            .map(|cell| cell.value.read())
    }

    /// Gets a write guard to an object.
    pub fn get_mut(&self, gc_ref: GcRef) -> Option<parking_lot::RwLockWriteGuard<'_, JsObject>> {
        self.objects
            .get(gc_ref.0)
            .and_then(|opt| opt.as_ref())
            .map(|cell| cell.value.write())
    }

    /// Adds a root reference.
    pub fn add_root(&mut self, gc_ref: GcRef) {
        if !self.roots.contains(&gc_ref) {
            self.roots.push(gc_ref);
        }
    }

    /// Removes a root reference.
    pub fn remove_root(&mut self, gc_ref: GcRef) {
        self.roots.retain(|r| *r != gc_ref);
    }

    /// Clears all root references.
    pub fn clear_roots(&mut self) {
        self.roots.clear();
    }

    /// Runs garbage collection.
    ///
    /// Uses parallel marking if the heap size exceeds the parallel threshold.
    pub fn collect(&mut self) {
        self.stats.collections += 1;
        self.allocations_since_gc = 0;

        // Choose marking strategy based on heap size
        #[cfg(feature = "parallel")]
        if self.objects.len() >= self.parallel_threshold {
            self.mark_phase_parallel();
        } else {
            self.mark_phase_sequential();
        }

        #[cfg(not(feature = "parallel"))]
        self.mark_phase_sequential();

        // Sweep phase (always sequential)
        self.sweep_phase();
    }

    /// Sequential mark phase for small heaps.
    fn mark_phase_sequential(&self) {
        // Reset all marks
        for obj in self.objects.iter().flatten() {
            obj.marked.store(false, Ordering::Relaxed);
        }

        // Mark from roots
        for root in &self.roots {
            self.mark_recursive(*root);
        }
    }

    /// Parallel mark phase for large heaps.
    #[cfg(feature = "parallel")]
    fn mark_phase_parallel(&self) {
        // Reset all marks in parallel
        self.objects.par_iter().flatten().for_each(|cell| {
            cell.marked.store(false, Ordering::Relaxed);
        });

        // Initial mark from roots (parallel)
        self.roots.par_iter().for_each(|root| {
            self.mark_recursive(*root);
        });
    }

    /// Recursively marks an object and its references.
    fn mark_recursive(&self, gc_ref: GcRef) {
        if let Some(Some(cell)) = self.objects.get(gc_ref.0) {
            // Try to mark - if already marked, return early
            if cell
                .marked
                .compare_exchange(false, true, Ordering::AcqRel, Ordering::Relaxed)
                .is_err()
            {
                return; // Already marked by another thread or earlier
            }

            // Trace references from this object
            let object = cell.value.read();
            if let Some(proto) = object.prototype {
                self.mark_recursive(proto);
            }
            for value in object.properties.values() {
                if let PropertyValue::Object(obj_ref) = value {
                    self.mark_recursive(*obj_ref);
                }
            }
        }
    }

    fn sweep_phase(&mut self) {
        let mut freed = 0;
        for i in 0..self.objects.len() {
            if let Some(cell) = &self.objects[i] {
                if !cell.marked.load(Ordering::Relaxed) {
                    // Object is unreachable, free it
                    self.objects[i] = None;
                    self.free_list.push(i);
                    freed += 1;
                }
            }
        }
        self.stats.total_freed += freed;
        self.stats.live_objects -= freed;
    }

    /// Returns GC statistics.
    pub fn stats(&self) -> &GcStats {
        &self.stats
    }

    /// Sets the GC threshold.
    pub fn set_threshold(&mut self, threshold: usize) {
        self.gc_threshold = threshold;
    }

    /// Sets the parallel marking threshold.
    ///
    /// When the heap contains more objects than this threshold,
    /// the mark phase will run in parallel.
    pub fn set_parallel_threshold(&mut self, threshold: usize) {
        self.parallel_threshold = threshold;
    }

    /// Returns the number of live objects.
    pub fn len(&self) -> usize {
        self.stats.live_objects
    }

    /// Returns whether the heap is empty.
    pub fn is_empty(&self) -> bool {
        self.stats.live_objects == 0
    }
}

impl Default for Heap {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allocate_and_get() {
        let mut heap = Heap::new();
        let obj_ref = heap.allocate(JsObject::new());

        assert!(heap.get(obj_ref).is_some());
    }

    #[test]
    fn test_gc_collects_unreachable() {
        let mut heap = Heap::new();
        heap.set_threshold(1000); // Disable auto-GC

        // Allocate without rooting
        let _obj_ref = heap.allocate(JsObject::new());

        assert_eq!(heap.stats().live_objects, 1);

        heap.collect();

        assert_eq!(heap.stats().live_objects, 0);
        assert_eq!(heap.stats().total_freed, 1);
    }

    #[test]
    fn test_gc_keeps_rooted() {
        let mut heap = Heap::new();
        heap.set_threshold(1000);

        let obj_ref = heap.allocate(JsObject::new());
        heap.add_root(obj_ref);

        heap.collect();

        assert_eq!(heap.stats().live_objects, 1);
        assert!(heap.get(obj_ref).is_some());
    }

    #[test]
    fn test_gc_traces_references() {
        let mut heap = Heap::new();
        heap.set_threshold(1000);

        // Create two objects, one referencing the other
        let child_ref = heap.allocate(JsObject::new());
        let mut parent = JsObject::new();
        parent.set("child".to_string(), PropertyValue::Object(child_ref));
        let parent_ref = heap.allocate(parent);

        // Only root the parent
        heap.add_root(parent_ref);

        heap.collect();

        // Both should still be alive
        assert_eq!(heap.stats().live_objects, 2);
        assert!(heap.get(parent_ref).is_some());
        assert!(heap.get(child_ref).is_some());
    }

    #[cfg(feature = "parallel")]
    #[test]
    fn test_parallel_gc() {
        let mut heap = Heap::new();
        heap.set_threshold(10000);
        heap.set_parallel_threshold(10); // Low threshold to test parallel path

        // Create many objects
        let mut last_ref = heap.allocate(JsObject::new());
        heap.add_root(last_ref);

        for i in 0..100 {
            let mut obj = JsObject::new();
            obj.set(format!("index"), PropertyValue::Number(i as f64));
            obj.set("prev".to_string(), PropertyValue::Object(last_ref));
            let new_ref = heap.allocate(obj);
            heap.remove_root(last_ref);
            heap.add_root(new_ref);
            last_ref = new_ref;
        }

        assert_eq!(heap.stats().live_objects, 101);

        heap.collect();

        // Only the chain from root should survive
        assert!(heap.stats().live_objects > 0);
    }
}
