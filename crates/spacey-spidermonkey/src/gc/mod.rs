//! Garbage collector for the JavaScript runtime.
//!
//! This module implements a simple mark-and-sweep garbage collector.
//!
//! ## Architecture
//!
//! The GC consists of:
//! - A heap of `GcObject` instances
//! - A set of root references
//! - Mark-and-sweep collection algorithm
//!
//! ## Usage
//!
//! ```ignore
//! let mut heap = Heap::new();
//! let obj_ref = heap.allocate(JsObject::new());
//! // ... use obj_ref ...
//! heap.collect(); // Run garbage collection
//! ```

use std::cell::Cell;
use std::collections::HashMap;

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
pub trait GcTrace {
    /// Mark all references held by this object.
    fn trace(&self, tracer: &mut Tracer);
}

/// Helper for tracing object references.
pub struct Tracer<'a> {
    heap: &'a mut Heap,
}

impl<'a> Tracer<'a> {
    /// Mark a GC reference as reachable.
    pub fn mark(&mut self, gc_ref: GcRef) {
        self.heap.mark(gc_ref);
    }
}

/// A garbage-collected heap object wrapper.
struct GcCell<T> {
    /// The object data
    value: T,
    /// Whether this object has been marked during GC
    marked: Cell<bool>,
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
    fn trace(&self, tracer: &mut Tracer) {
        if let Some(proto) = self.prototype {
            tracer.mark(proto);
        }
        for value in self.properties.values() {
            if let PropertyValue::Object(obj_ref) = value {
                tracer.mark(*obj_ref);
            }
        }
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
            value: object,
            marked: Cell::new(false),
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

    /// Gets a reference to an object.
    pub fn get(&self, gc_ref: GcRef) -> Option<&JsObject> {
        self.objects
            .get(gc_ref.0)
            .and_then(|opt| opt.as_ref())
            .map(|cell| &cell.value)
    }

    /// Gets a mutable reference to an object.
    pub fn get_mut(&mut self, gc_ref: GcRef) -> Option<&mut JsObject> {
        self.objects
            .get_mut(gc_ref.0)
            .and_then(|opt| opt.as_mut())
            .map(|cell| &mut cell.value)
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
    pub fn collect(&mut self) {
        self.stats.collections += 1;
        self.allocations_since_gc = 0;

        // Mark phase
        self.mark_phase();

        // Sweep phase
        self.sweep_phase();
    }

    fn mark_phase(&mut self) {
        // Reset all marks
        for obj in self.objects.iter().flatten() {
            obj.marked.set(false);
        }

        // Mark from roots
        let roots: Vec<GcRef> = self.roots.clone();
        for root in roots {
            self.mark(root);
        }
    }

    fn mark(&mut self, gc_ref: GcRef) {
        if let Some(Some(cell)) = self.objects.get(gc_ref.0) {
            if cell.marked.get() {
                return; // Already marked
            }
            cell.marked.set(true);

            // Trace references from this object
            let object = cell.value.clone();
            let mut tracer = Tracer { heap: self };
            object.trace(&mut tracer);
        }
    }

    fn sweep_phase(&mut self) {
        for i in 0..self.objects.len() {
            if let Some(cell) = &self.objects[i] {
                if !cell.marked.get() {
                    // Object is unreachable, free it
                    self.objects[i] = None;
                    self.free_list.push(i);
                    self.stats.total_freed += 1;
                    self.stats.live_objects -= 1;
                }
            }
        }
    }

    /// Returns GC statistics.
    pub fn stats(&self) -> &GcStats {
        &self.stats
    }

    /// Sets the GC threshold.
    pub fn set_threshold(&mut self, threshold: usize) {
        self.gc_threshold = threshold;
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
}
