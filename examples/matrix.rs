use anyhow::Result;
use concurrency::{multiply, Matrix};

fn main() -> Result<()> {
    let a = Matrix::new([1, 2, 3, 4, 5, 6], 2, 3);
    let b = Matrix::new([1, 2, 3, 4, 5, 6], 3, 2);
    let c = multiply(&a, &b)?;
    println!("matrix: {}", c);
    Ok(())
}
