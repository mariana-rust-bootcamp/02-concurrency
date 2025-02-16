use std::ops::{AddAssign, Deref, Mul};

use anyhow::{anyhow, Result};

pub struct Vector<T> {
    data: Vec<T>,
}

// pretend this is a heavy operation, cpu intensive
pub fn dot_product<T>(a: Vector<T>, b: Vector<T>) -> Result<T>
where
    T: Copy + Default + Mul<Output = T> + AddAssign,
{
    if a.len() != b.len() {
        return Err(anyhow!("Dot product error; a.len != b.len"));
    }

    let mut sum = T::default();
    for i in 0..a.len() {
        sum += a[i] * b[i];
    }

    Ok(sum)
}

impl<T> Deref for Vector<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> Vector<T> {
    pub fn new(data: impl Into<Vec<T>>) -> Self {
        Self { data: data.into() }
    }
}
