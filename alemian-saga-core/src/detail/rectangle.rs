use crate::*;

// Represents a rectangle
pub struct Rectangle<T> {
    pub top_left: Vector<T>,
    pub size: Vector<T>,
}

impl<T: std::ops::Add<Output = T> + Copy> Rectangle<T> {
    pub fn top(&self) -> T {
        self.top_left.y
    }
    pub fn left(&self) -> T {
        self.top_left.x
    }
    pub fn width(&self) -> T {
        self.size.x
    }
    pub fn height(&self) -> T {
        self.size.y
    }
    pub fn bottom(&self) -> T {
        self.top() + self.height()
    }
    pub fn right(&self) -> T {
        self.left() + self.width()
    }
}
