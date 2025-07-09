//! Mathematical utilities and structures

/// 2D vector structure
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    /// Create a new 2D vector
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
    
    /// Zero vector
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };
    
    /// Unit vector along X axis
    pub const X: Self = Self { x: 1.0, y: 0.0 };
    
    /// Unit vector along Y axis
    pub const Y: Self = Self { x: 0.0, y: 1.0 };
}

impl Default for Vec2 {
    fn default() -> Self {
        Self::ZERO
    }
}

/// Rectangle structure for bounds and hit testing
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rectangle {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rectangle {
    /// Create a new rectangle
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }
}

/// 2D transformation matrix
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform2D {
    // Placeholder - will be implemented in future stories
    pub translation: Vec2,
    pub rotation: f32,
    pub scale: Vec2,
}

impl Transform2D {
    /// Create identity transform
    pub fn identity() -> Self {
        Self {
            translation: Vec2::ZERO,
            rotation: 0.0,
            scale: Vec2::new(1.0, 1.0),
        }
    }
}

impl Default for Transform2D {
    fn default() -> Self {
        Self::identity()
    }
}

/// 4x4 matrix for 3D transformations and projections
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mat4 {
    /// Matrix data stored in column-major order
    pub data: [[f32; 4]; 4],
}

impl Mat4 {
    /// Create a new matrix from array data
    pub fn new(data: [[f32; 4]; 4]) -> Self {
        Self { data }
    }

    /// Create an identity matrix
    pub fn identity() -> Self {
        Self {
            data: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ]
        }
    }

    /// Create an orthographic projection matrix
    /// 
    /// # Arguments
    /// * `left` - Left boundary of the view frustum
    /// * `right` - Right boundary of the view frustum  
    /// * `bottom` - Bottom boundary of the view frustum
    /// * `top` - Top boundary of the view frustum
    /// * `near` - Near clipping plane distance
    /// * `far` - Far clipping plane distance
    pub fn orthographic(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> Self {
        let width = right - left;
        let height = top - bottom;
        let depth = far - near;

        Self {
            data: [
                [2.0 / width, 0.0, 0.0, 0.0],
                [0.0, 2.0 / height, 0.0, 0.0],
                [0.0, 0.0, -2.0 / depth, 0.0],
                [-(right + left) / width, -(top + bottom) / height, -(far + near) / depth, 1.0],
            ]
        }
    }

    /// Create a translation matrix
    pub fn translation(x: f32, y: f32, z: f32) -> Self {
        Self {
            data: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [x, y, z, 1.0],
            ]
        }
    }

    /// Create a scale matrix
    pub fn scale(x: f32, y: f32, z: f32) -> Self {
        Self {
            data: [
                [x, 0.0, 0.0, 0.0],
                [0.0, y, 0.0, 0.0],
                [0.0, 0.0, z, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ]
        }
    }

    /// Get matrix element at [row][col]
    pub fn get(&self, row: usize, col: usize) -> f32 {
        self.data[col][row] // Column-major storage
    }

    /// Set matrix element at [row][col]
    pub fn set(&mut self, row: usize, col: usize, value: f32) {
        self.data[col][row] = value; // Column-major storage
    }

    /// Convert to a flattened array in column-major order (for GPU upload)
    pub fn as_array(&self) -> [f32; 16] {
        [
            self.data[0][0], self.data[0][1], self.data[0][2], self.data[0][3],
            self.data[1][0], self.data[1][1], self.data[1][2], self.data[1][3],
            self.data[2][0], self.data[2][1], self.data[2][2], self.data[2][3],
            self.data[3][0], self.data[3][1], self.data[3][2], self.data[3][3],
        ]
    }

    /// Transform a 2D point using this matrix (assuming z=0, w=1)
    pub fn transform_point_2d(&self, point: Vec2) -> Vec2 {
        let x = self.data[0][0] * point.x + self.data[1][0] * point.y + self.data[3][0];
        let y = self.data[0][1] * point.x + self.data[1][1] * point.y + self.data[3][1];
        Vec2::new(x, y)
    }
}

impl Default for Mat4 {
    fn default() -> Self {
        Self::identity()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vec2_creation() {
        let v = Vec2::new(3.0, 4.0);
        assert_eq!(v.x, 3.0);
        assert_eq!(v.y, 4.0);
    }

    #[test]
    fn test_vec2_constants() {
        assert_eq!(Vec2::ZERO, Vec2::new(0.0, 0.0));
        assert_eq!(Vec2::X, Vec2::new(1.0, 0.0));
        assert_eq!(Vec2::Y, Vec2::new(0.0, 1.0));
    }

    #[test]
    fn test_mat4_identity() {
        let identity = Mat4::identity();
        
        // Check diagonal elements
        for i in 0..4 {
            assert_eq!(identity.get(i, i), 1.0);
        }
        
        // Check off-diagonal elements
        for i in 0..4 {
            for j in 0..4 {
                if i != j {
                    assert_eq!(identity.get(i, j), 0.0);
                }
            }
        }
    }

    #[test]
    fn test_mat4_orthographic() {
        let ortho = Mat4::orthographic(-1.0, 1.0, -1.0, 1.0, -1.0, 1.0);
        
        // For a standard orthographic projection with bounds (-1,1), 
        // the scaling factors should be 1.0
        assert_eq!(ortho.get(0, 0), 1.0); // 2.0 / (1.0 - (-1.0)) = 1.0
        assert_eq!(ortho.get(1, 1), 1.0); // 2.0 / (1.0 - (-1.0)) = 1.0
        assert_eq!(ortho.get(2, 2), -1.0); // -2.0 / (1.0 - (-1.0)) = -1.0
        assert_eq!(ortho.get(3, 3), 1.0);
    }

    #[test]
    fn test_mat4_translation() {
        let translation = Mat4::translation(10.0, 20.0, 30.0);
        
        // Translation values should be in the last column
        assert_eq!(translation.get(0, 3), 10.0);
        assert_eq!(translation.get(1, 3), 20.0);
        assert_eq!(translation.get(2, 3), 30.0);
        assert_eq!(translation.get(3, 3), 1.0);
        
        // Diagonal should be 1.0
        assert_eq!(translation.get(0, 0), 1.0);
        assert_eq!(translation.get(1, 1), 1.0);
        assert_eq!(translation.get(2, 2), 1.0);
    }

    #[test]
    fn test_mat4_scale() {
        let scale = Mat4::scale(2.0, 3.0, 4.0);
        
        // Scale values should be on the diagonal
        assert_eq!(scale.get(0, 0), 2.0);
        assert_eq!(scale.get(1, 1), 3.0);
        assert_eq!(scale.get(2, 2), 4.0);
        assert_eq!(scale.get(3, 3), 1.0);
        
        // Off-diagonal should be zero
        assert_eq!(scale.get(0, 1), 0.0);
        assert_eq!(scale.get(1, 0), 0.0);
    }

    #[test]
    fn test_mat4_get_set() {
        let mut matrix = Mat4::identity();
        
        matrix.set(1, 2, 42.0);
        assert_eq!(matrix.get(1, 2), 42.0);
        
        matrix.set(3, 0, -17.5);
        assert_eq!(matrix.get(3, 0), -17.5);
    }

    #[test]
    fn test_mat4_as_array() {
        let matrix = Mat4::identity();
        let array = matrix.as_array();
        
        // Identity matrix should have 1.0 at positions 0, 5, 10, 15 (diagonal)
        assert_eq!(array[0], 1.0);   // [0,0]
        assert_eq!(array[5], 1.0);   // [1,1]
        assert_eq!(array[10], 1.0);  // [2,2]
        assert_eq!(array[15], 1.0);  // [3,3]
        
        // All other positions should be 0.0
        for (i, &value) in array.iter().enumerate() {
            if i != 0 && i != 5 && i != 10 && i != 15 {
                assert_eq!(value, 0.0);
            }
        }
    }

    #[test]
    fn test_mat4_transform_point_2d() {
        // Test with identity matrix
        let identity = Mat4::identity();
        let point = Vec2::new(5.0, 7.0);
        let transformed = identity.transform_point_2d(point);
        assert_eq!(transformed, point);
        
        // Test with translation
        let translation = Mat4::translation(10.0, 20.0, 0.0);
        let translated = translation.transform_point_2d(point);
        assert_eq!(translated, Vec2::new(15.0, 27.0));
        
        // Test with scale
        let scale = Mat4::scale(2.0, 3.0, 1.0);
        let scaled = scale.transform_point_2d(point);
        assert_eq!(scaled, Vec2::new(10.0, 21.0));
    }

    #[test]
    fn test_rectangle_creation() {
        let rect = Rectangle::new(10.0, 20.0, 100.0, 200.0);
        assert_eq!(rect.x, 10.0);
        assert_eq!(rect.y, 20.0);
        assert_eq!(rect.width, 100.0);
        assert_eq!(rect.height, 200.0);
    }

    #[test]
    fn test_transform2d_identity() {
        let transform = Transform2D::identity();
        assert_eq!(transform.translation, Vec2::ZERO);
        assert_eq!(transform.rotation, 0.0);
        assert_eq!(transform.scale, Vec2::new(1.0, 1.0));
    }
}