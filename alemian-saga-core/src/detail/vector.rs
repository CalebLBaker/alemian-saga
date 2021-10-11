use crate::*;

impl<T: num_traits::ToPrimitive> Vector<T> {
    pub fn lossy_cast<U: num_traits::NumCast>(self) -> Option<Vector<U>> {
        Some(Vector {
            x: U::from(self.x)?,
            y: U::from(self.y)?,
        })
    }
}

impl<T> Vector<T> {
    pub fn cast<U: From<T>>(self) -> Vector<U> {
        Vector {
            x: self.x.into(),
            y: self.y.into(),
        }
    }
}

impl<T: std::ops::Div> Vector<T> {
    pub fn piecewise_divide(self, rhs: Vector<T>) -> Vector<T::Output> {
        Vector {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
        }
    }
}

impl<T: std::ops::Mul> Vector<T> {
    pub fn piecewise_multiply(self, rhs: Vector<T>) -> Vector<T::Output> {
        Vector {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
        }
    }
}

impl<T: std::ops::Add<Output = T>> std::ops::Add for Vector<T> {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl<T: std::ops::Sub<Output = T>> std::ops::Sub for Vector<T> {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl<T: Copy + std::ops::Div<Output = T>> std::ops::Div<T> for Vector<T> {
    type Output = Self;
    fn div(self, rhs: T) -> Self::Output {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

impl Vector<crate::numeric_types::MapDistance> {
    pub fn lossy_cast<U: num_traits::NumCast>(self) -> Option<Vector<U>> {
        Some(Vector {
            x: U::from(self.x.value)?,
            y: U::from(self.y.value)?,
        })
    }
    pub fn from(source: Vector<i32>) -> Self {
        Vector {
            x: numeric_types::map_dist(source.x),
            y: numeric_types::map_dist(source.y),
        }
    }
}
