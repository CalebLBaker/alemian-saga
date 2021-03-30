use crate::*;

impl<T: Scalar + num_traits::ToPrimitive> Vector<T> {
    pub fn lossy_cast<U: num_traits::NumCast>(self) -> Option<Vector<U>> {
        Some(Vector {
            x: U::from(self.x)?,
            y: U::from(self.y)?,
        })
    }
}

impl<T: Scalar> Vector<T> {
    pub fn piecewise_divide<U: Scalar + Into<T>>(self, rhs: Vector<U>) -> Vector<T> {
        Vector {
            x: self.x / rhs.x.into(),
            y: self.y / rhs.y.into(),
        }
    }
    pub fn piecewise_multiply<U: Scalar + Into<T>>(self, rhs: Vector<U>) -> Vector<T> {
        Vector {
            x: self.x * rhs.x.into(),
            y: self.y * rhs.y.into(),
        }
    }
    pub fn cast<U: Scalar + From<T>>(self) -> Vector<U> {
        Vector {
            x: self.x.into(),
            y: self.y.into(),
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

impl<T: std::ops::Div<Output = T> + Copy> std::ops::Div<T> for Vector<T> {
    type Output = Self;
    fn div(self, rhs: T) -> Self::Output {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

