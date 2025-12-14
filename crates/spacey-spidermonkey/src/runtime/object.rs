//! JavaScript object representation.

use super::value::Value;
use rustc_hash::FxHashMap;

/// A JavaScript object.
#[derive(Debug, Clone)]
pub struct Object {
    /// The prototype of this object
    pub prototype: Option<Box<Object>>,
    /// The properties
    pub properties: FxHashMap<String, Property>,
    /// Whether the object is extensible
    pub extensible: bool,
}

impl Object {
    /// Creates a new empty object.
    pub fn new() -> Self {
        Self {
            prototype: None,
            properties: FxHashMap::default(),
            extensible: true,
        }
    }

    /// Gets a property value.
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.properties.get(key).map(|p| &p.value)
    }

    /// Sets a property value.
    pub fn set(&mut self, key: String, value: Value) {
        self.properties.insert(
            key,
            Property {
                value,
                writable: true,
                enumerable: true,
                configurable: true,
            },
        );
    }

    /// Deletes a property.
    pub fn delete(&mut self, key: &str) -> bool {
        if let Some(prop) = self.properties.get(key) {
            if prop.configurable {
                self.properties.remove(key);
                return true;
            }
        }
        false
    }

    /// Checks if a property exists.
    pub fn has(&self, key: &str) -> bool {
        self.properties.contains_key(key)
    }
}

impl Default for Object {
    fn default() -> Self {
        Self::new()
    }
}

/// A property descriptor.
#[derive(Debug, Clone)]
pub struct Property {
    /// The property value
    pub value: Value,
    /// Whether the property is writable
    pub writable: bool,
    /// Whether the property is enumerable
    pub enumerable: bool,
    /// Whether the property is configurable
    pub configurable: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_object() {
        let obj = Object::new();
        assert!(obj.extensible);
        assert!(obj.prototype.is_none());
        assert!(!obj.has("x"));
    }

    #[test]
    fn test_default_object() {
        let obj = Object::default();
        assert!(obj.extensible);
        assert!(obj.prototype.is_none());
    }

    #[test]
    fn test_set_and_get() {
        let mut obj = Object::new();

        obj.set("x".to_string(), Value::Number(42.0));

        assert!(obj.has("x"));
        assert_eq!(obj.get("x"), Some(&Value::Number(42.0)));
    }

    #[test]
    fn test_get_nonexistent() {
        let obj = Object::new();
        assert!(obj.get("x").is_none());
    }

    #[test]
    fn test_overwrite_property() {
        let mut obj = Object::new();

        obj.set("x".to_string(), Value::Number(1.0));
        obj.set("x".to_string(), Value::Number(2.0));

        assert_eq!(obj.get("x"), Some(&Value::Number(2.0)));
    }

    #[test]
    fn test_delete_configurable() {
        let mut obj = Object::new();

        obj.set("x".to_string(), Value::Number(42.0));
        assert!(obj.has("x"));

        // Default properties are configurable
        assert!(obj.delete("x"));
        assert!(!obj.has("x"));
    }

    #[test]
    fn test_delete_non_configurable() {
        let mut obj = Object::new();

        // Manually insert a non-configurable property
        obj.properties.insert(
            "x".to_string(),
            Property {
                value: Value::Number(42.0),
                writable: true,
                enumerable: true,
                configurable: false,
            },
        );

        // Cannot delete non-configurable property
        assert!(!obj.delete("x"));
        assert!(obj.has("x"));
    }

    #[test]
    fn test_delete_nonexistent() {
        let mut obj = Object::new();

        // Deleting nonexistent property returns false
        assert!(!obj.delete("x"));
    }

    #[test]
    fn test_has() {
        let mut obj = Object::new();

        assert!(!obj.has("x"));
        obj.set("x".to_string(), Value::Undefined);
        assert!(obj.has("x")); // Even undefined counts as having the property
    }

    #[test]
    fn test_multiple_properties() {
        let mut obj = Object::new();

        obj.set("a".to_string(), Value::Number(1.0));
        obj.set("b".to_string(), Value::String("hello".to_string()));
        obj.set("c".to_string(), Value::Boolean(true));
        obj.set("d".to_string(), Value::Null);

        assert_eq!(obj.get("a"), Some(&Value::Number(1.0)));
        assert_eq!(obj.get("b"), Some(&Value::String("hello".to_string())));
        assert_eq!(obj.get("c"), Some(&Value::Boolean(true)));
        assert_eq!(obj.get("d"), Some(&Value::Null));
    }

    #[test]
    fn test_property_descriptor_defaults() {
        let mut obj = Object::new();
        obj.set("x".to_string(), Value::Number(42.0));

        let prop = obj.properties.get("x").unwrap();
        assert!(prop.writable);
        assert!(prop.enumerable);
        assert!(prop.configurable);
    }

    #[test]
    fn test_object_clone() {
        let mut obj = Object::new();
        obj.set("x".to_string(), Value::Number(42.0));

        let cloned = obj.clone();
        assert_eq!(cloned.get("x"), Some(&Value::Number(42.0)));
    }

    #[test]
    fn test_object_with_prototype() {
        let mut proto = Object::new();
        proto.set("inherited".to_string(), Value::Number(100.0));

        let obj = Object {
            prototype: Some(Box::new(proto)),
            properties: FxHashMap::default(),
            extensible: true,
        };

        assert!(obj.prototype.is_some());
    }

    #[test]
    fn test_extensible_flag() {
        let mut obj = Object::new();
        assert!(obj.extensible);

        obj.extensible = false;
        assert!(!obj.extensible);

        // Note: Current implementation doesn't check extensible in set()
        // This is a placeholder for when that's implemented
    }

    #[test]
    fn test_property_debug() {
        let prop = Property {
            value: Value::Number(42.0),
            writable: true,
            enumerable: true,
            configurable: true,
        };
        let debug_str = format!("{:?}", prop);
        assert!(debug_str.contains("42"));
    }

    #[test]
    fn test_object_debug() {
        let mut obj = Object::new();
        obj.set("x".to_string(), Value::Number(42.0));
        let debug_str = format!("{:?}", obj);
        assert!(debug_str.contains("properties"));
    }
}
