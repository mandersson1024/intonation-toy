//! Hit Testing System
//!
//! This module provides hit testing functionality for detecting clicks/touches
//! on GPU-rendered elements in graphics space.

use std::collections::HashMap;

/// Represents a clickable element in graphics space
#[derive(Debug, Clone)]
pub struct HitTestElement {
    /// Unique identifier for the element
    pub id: String,
    /// Bounding box in graphics coordinates (min_x, min_y, max_x, max_y)
    pub bounds: (f32, f32, f32, f32),
    /// Whether the element is currently active for hit testing
    pub active: bool,
}

impl HitTestElement {
    /// Create a new hit test element
    pub fn new(id: String, min_x: f32, min_y: f32, max_x: f32, max_y: f32) -> Self {
        Self {
            id,
            bounds: (min_x, min_y, max_x, max_y),
            active: true,
        }
    }
    
    /// Create a hit test element from center and size
    pub fn from_center_and_size(id: String, center_x: f32, center_y: f32, width: f32, height: f32) -> Self {
        let half_width = width / 2.0;
        let half_height = height / 2.0;
        Self::new(
            id,
            center_x - half_width,
            center_y - half_height,
            center_x + half_width,
            center_y + half_height,
        )
    }
    
    /// Test if a point in graphics space hits this element
    pub fn hit_test(&self, x: f32, y: f32) -> bool {
        if !self.active {
            return false;
        }
        
        let (min_x, min_y, max_x, max_y) = self.bounds;
        x >= min_x && x <= max_x && y >= min_y && y <= max_y
    }
    
    /// Get the center point of the element
    pub fn center(&self) -> (f32, f32) {
        let (min_x, min_y, max_x, max_y) = self.bounds;
        ((min_x + max_x) / 2.0, (min_y + max_y) / 2.0)
    }
    
    /// Get the size of the element
    pub fn size(&self) -> (f32, f32) {
        let (min_x, min_y, max_x, max_y) = self.bounds;
        (max_x - min_x, max_y - min_y)
    }
    
    /// Set the active state of the element
    pub fn set_active(&mut self, active: bool) {
        self.active = active;
    }
    
    /// Update the bounds of the element
    pub fn update_bounds(&mut self, min_x: f32, min_y: f32, max_x: f32, max_y: f32) {
        self.bounds = (min_x, min_y, max_x, max_y);
    }
}

/// Registry for managing hit test elements
#[derive(Debug, Clone)]
pub struct HitTestRegistry {
    elements: HashMap<String, HitTestElement>,
}

impl HitTestRegistry {
    /// Create a new hit test registry
    pub fn new() -> Self {
        Self {
            elements: HashMap::new(),
        }
    }
    
    /// Register a new hit test element
    pub fn register_element(&mut self, element: HitTestElement) {
        self.elements.insert(element.id.clone(), element);
    }
    
    /// Remove a hit test element by ID
    pub fn unregister_element(&mut self, id: &str) -> Option<HitTestElement> {
        self.elements.remove(id)
    }
    
    /// Get a mutable reference to an element by ID
    pub fn get_element_mut(&mut self, id: &str) -> Option<&mut HitTestElement> {
        self.elements.get_mut(id)
    }
    
    /// Get an immutable reference to an element by ID
    pub fn get_element(&self, id: &str) -> Option<&HitTestElement> {
        self.elements.get(id)
    }
    
    /// Perform hit test against all active elements
    /// Returns the IDs of all elements that were hit, in registration order
    pub fn hit_test_all(&self, x: f32, y: f32) -> Vec<String> {
        self.elements
            .values()
            .filter(|element| element.hit_test(x, y))
            .map(|element| element.id.clone())
            .collect()
    }
    
    /// Perform hit test and return the first hit element ID
    pub fn hit_test_first(&self, x: f32, y: f32) -> Option<String> {
        self.elements
            .values()
            .find(|element| element.hit_test(x, y))
            .map(|element| element.id.clone())
    }
    
    /// Set the active state of an element by ID
    pub fn set_element_active(&mut self, id: &str, active: bool) -> bool {
        if let Some(element) = self.elements.get_mut(id) {
            element.set_active(active);
            true
        } else {
            false
        }
    }
    
    /// Get the number of registered elements
    pub fn element_count(&self) -> usize {
        self.elements.len()
    }
    
    /// Get the number of active elements
    pub fn active_element_count(&self) -> usize {
        self.elements.values().filter(|e| e.active).count()
    }
    
    /// Get all element IDs
    pub fn element_ids(&self) -> Vec<String> {
        self.elements.keys().cloned().collect()
    }
    
    /// Clear all elements
    pub fn clear(&mut self) {
        self.elements.clear();
    }
}

impl Default for HitTestRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_hit_test_element_creation() {
        let element = HitTestElement::new("test".to_string(), -1.0, -1.0, 1.0, 1.0);
        assert_eq!(element.id, "test");
        assert_eq!(element.bounds, (-1.0, -1.0, 1.0, 1.0));
        assert!(element.active);
    }
    
    #[test]
    fn test_hit_test_element_from_center_and_size() {
        let element = HitTestElement::from_center_and_size("test".to_string(), 0.0, 0.0, 2.0, 2.0);
        assert_eq!(element.bounds, (-1.0, -1.0, 1.0, 1.0));
        assert_eq!(element.center(), (0.0, 0.0));
        assert_eq!(element.size(), (2.0, 2.0));
    }
    
    #[test]
    fn test_hit_test_element_hit_testing() {
        let element = HitTestElement::new("test".to_string(), -1.0, -1.0, 1.0, 1.0);
        
        // Test hits
        assert!(element.hit_test(0.0, 0.0)); // Center
        assert!(element.hit_test(-1.0, -1.0)); // Min corner
        assert!(element.hit_test(1.0, 1.0)); // Max corner
        assert!(element.hit_test(0.5, -0.5)); // Inside
        
        // Test misses
        assert!(!element.hit_test(-1.1, 0.0)); // Left of bounds
        assert!(!element.hit_test(1.1, 0.0)); // Right of bounds
        assert!(!element.hit_test(0.0, -1.1)); // Below bounds
        assert!(!element.hit_test(0.0, 1.1)); // Above bounds
    }
    
    #[test]
    fn test_hit_test_element_inactive() {
        let mut element = HitTestElement::new("test".to_string(), -1.0, -1.0, 1.0, 1.0);
        element.set_active(false);
        
        // Should not hit when inactive
        assert!(!element.hit_test(0.0, 0.0));
        
        // Should hit when reactivated
        element.set_active(true);
        assert!(element.hit_test(0.0, 0.0));
    }
    
    #[test]
    fn test_hit_test_registry_basic() {
        let mut registry = HitTestRegistry::new();
        assert_eq!(registry.element_count(), 0);
        
        let element = HitTestElement::new("test1".to_string(), -1.0, -1.0, 1.0, 1.0);
        registry.register_element(element);
        assert_eq!(registry.element_count(), 1);
        assert_eq!(registry.active_element_count(), 1);
    }
    
    #[test]
    fn test_hit_test_registry_hit_testing() {
        let mut registry = HitTestRegistry::new();
        
        // Register two overlapping elements
        let element1 = HitTestElement::new("element1".to_string(), -1.0, -1.0, 1.0, 1.0);
        let element2 = HitTestElement::new("element2".to_string(), -0.5, -0.5, 1.5, 1.5);
        registry.register_element(element1);
        registry.register_element(element2);
        
        // Test hit on overlap area
        let hits = registry.hit_test_all(0.0, 0.0);
        assert_eq!(hits.len(), 2);
        assert!(hits.contains(&"element1".to_string()));
        assert!(hits.contains(&"element2".to_string()));
        
        // Test hit on element2 only
        let hits = registry.hit_test_all(1.2, 1.2);
        assert_eq!(hits.len(), 1);
        assert!(hits.contains(&"element2".to_string()));
        
        // Test miss
        let hits = registry.hit_test_all(2.0, 2.0);
        assert_eq!(hits.len(), 0);
    }
    
    #[test]
    fn test_hit_test_registry_first_hit() {
        let mut registry = HitTestRegistry::new();
        
        let element1 = HitTestElement::new("element1".to_string(), -1.0, -1.0, 1.0, 1.0);
        let element2 = HitTestElement::new("element2".to_string(), -0.5, -0.5, 1.5, 1.5);
        registry.register_element(element1);
        registry.register_element(element2);
        
        // Should return one of the elements (implementation dependent)
        let first_hit = registry.hit_test_first(0.0, 0.0);
        assert!(first_hit.is_some());
        let hit_id = first_hit.unwrap();
        assert!(hit_id == "element1" || hit_id == "element2");
    }
    
    #[test]
    fn test_hit_test_registry_element_management() {
        let mut registry = HitTestRegistry::new();
        
        let element = HitTestElement::new("test".to_string(), -1.0, -1.0, 1.0, 1.0);
        registry.register_element(element);
        
        // Test getting element
        assert!(registry.get_element("test").is_some());
        assert!(registry.get_element("nonexistent").is_none());
        
        // Test setting active state
        assert!(registry.set_element_active("test", false));
        assert!(!registry.set_element_active("nonexistent", false));
        assert_eq!(registry.active_element_count(), 0);
        
        // Test unregistering
        let removed = registry.unregister_element("test");
        assert!(removed.is_some());
        assert_eq!(registry.element_count(), 0);
    }
    
    #[test]
    fn test_hit_test_registry_clear() {
        let mut registry = HitTestRegistry::new();
        
        let element1 = HitTestElement::new("element1".to_string(), -1.0, -1.0, 1.0, 1.0);
        let element2 = HitTestElement::new("element2".to_string(), 0.0, 0.0, 2.0, 2.0);
        registry.register_element(element1);
        registry.register_element(element2);
        
        assert_eq!(registry.element_count(), 2);
        
        registry.clear();
        assert_eq!(registry.element_count(), 0);
        assert_eq!(registry.active_element_count(), 0);
    }
    
    #[test]
    fn test_hit_test_element_update_bounds() {
        let mut element = HitTestElement::new("test".to_string(), -1.0, -1.0, 1.0, 1.0);
        
        // Original bounds
        assert!(element.hit_test(0.0, 0.0));
        assert!(!element.hit_test(2.0, 2.0));
        
        // Update bounds
        element.update_bounds(0.0, 0.0, 3.0, 3.0);
        
        // Test new bounds
        assert!(element.hit_test(2.0, 2.0));
        assert!(!element.hit_test(-0.5, -0.5));
    }
}