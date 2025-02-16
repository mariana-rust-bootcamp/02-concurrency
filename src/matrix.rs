use std::{
    fmt::{self, Debug, Display},
    ops::{AddAssign, Mul},
    sync::mpsc,
    thread,
};

use anyhow::{anyhow, Result};

use crate::{dot_product, Vector};

// 线程池线程数
const NUM_THREADS: usize = 4;

pub struct Matrix<T> {
    data: Vec<T>,
    row: usize,
    col: usize,
}

/** 创建进程间通信的数据结构 [S] */
/// 将两个矩阵每次要计算的数放到row和col中
pub struct MsgInput<T> {
    idx: usize,
    row: Vector<T>,
    col: Vector<T>,
}

pub struct MsgOutput<T> {
    idx: usize,
    value: T,
}

pub struct Msg<T> {
    input: MsgInput<T>,
    sender: oneshot::Sender<MsgOutput<T>>,
}
/** 创建进程间通信的数据结构 [E] */
/// 矩阵加法/矩阵减法都要求两个矩阵结构都为M * N -> 计算结果也为M * N
/// 矩阵乘法要求两个矩阵为M * N 和 N * M -> 计算结果为M * M
pub fn multiply<T>(a: &Matrix<T>, b: &Matrix<T>) -> Result<Matrix<T>>
where
    // 对泛型的约束: 实现 +, *, +=, Copy, Default trait
    T: Copy + Default + Mul<Output = T> + AddAssign + Send + 'static,
{
    //
    if a.col != b.row {
        return Err(anyhow!("Matrix multiply error: a.col != b.row"));
    }

    // 创建线程池 这里的线程池是用于处理计算
    // 而每个计算结果是通过另外的oneshot通道通信的 两个线程作用不一样
    let senders = (0..NUM_THREADS)
        .map(|_| {
            let (tx, rx) = mpsc::channel::<Msg<T>>();
            thread::spawn(move || {
                for msg in rx {
                    let value = dot_product(msg.input.row, msg.input.col)?;
                    if let Err(e) = msg.sender.send(MsgOutput {
                        idx: msg.input.idx,
                        value,
                    }) {
                        eprintln!("Send error: {:?}", e);
                    }
                }
                Ok::<_, anyhow::Error>(())
            });
            tx
        })
        .collect::<Vec<_>>();

    let matrix_len = a.row * b.col;
    // 初始化一个相加后的数组
    let mut data = vec![T::default(); matrix_len];
    let mut receiver = Vec::with_capacity(matrix_len);

    // 矩阵相乘
    for i in 0..a.row {
        for j in 0..b.col {
            let row = Vector::new(&a.data[i * a.col..(i + 1) * a.col]);
            let col_data = b.data[j..]
                .iter()
                .step_by(b.col)
                .copied() // copied()很重要 iter()获得的是引用 使用copied()将引用转换为值
                .collect::<Vec<_>>();
            let col = Vector::new(col_data);
            let idx = i * b.col + j;
            let input = MsgInput::new(idx, row, col);
            let (tx, rx) = oneshot::channel();
            let msg = Msg::new(input, tx);
            if let Err(e) = senders[idx % NUM_THREADS].send(msg) {
                eprintln!("Send error: {:?}", e);
            }
            receiver.push(rx);
        }
    }

    for rx in receiver {
        let output = rx.recv()?;
        data[output.idx] = output.value;
    }

    Ok(Matrix::new(data, a.row, b.col))
}

impl<T> Matrix<T> {
    // 支持任何实现了Into Vec trait的类型
    pub fn new(data: impl Into<Vec<T>>, row: usize, col: usize) -> Self {
        Self {
            data: data.into(),
            row,
            col,
        }
    }
}

// 实现Matrix的Debug trait和Display trait用于打印矩阵结构
impl<T> Debug for Matrix<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Matrix(row={}, col={}, {})", self.row, self.col, self)
    }
}

impl<T> Display for Matrix<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{")?;
        for i in 0..self.row {
            for j in 0..self.col {
                write!(f, "{}", self.data[i * self.col + j])?;
                if j != self.col - 1 {
                    write!(f, " ")?;
                }
            }
            if i != self.row - 1 {
                write!(f, ", ")?;
            }
        }
        write!(f, "}}")?;
        Ok(())
    }
}

impl<T> MsgInput<T> {
    pub fn new(idx: usize, row: Vector<T>, col: Vector<T>) -> Self {
        Self { idx, row, col }
    }
}

impl<T> Msg<T> {
    pub fn new(input: MsgInput<T>, sender: oneshot::Sender<MsgOutput<T>>) -> Self {
        Self { input, sender }
    }
}

impl<T> Mul for Matrix<T>
where
    T: Copy + Default + Mul<Output = T> + AddAssign + Send + 'static,
{
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        multiply(&self, &rhs).expect("Matrix multiply error")
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Ok;

    use super::*;

    #[test]
    fn test_matrix_multiply() -> Result<()> {
        let a = Matrix::new([1, 2, 3, 4, 5, 6], 2, 3);
        let b = Matrix::new([1, 2, 3, 4, 5, 6], 3, 2);
        let c = a * b;
        assert_eq!(c.row, 2);
        assert_eq!(c.col, 2);
        assert_eq!(c.data, vec![22, 28, 49, 64]);
        assert_eq!(format!("{:?}", c), "Matrix(row=2, col=2, {22 28, 49 64})");

        Ok(())
    }

    #[test]
    fn test_matrix_display() -> Result<()> {
        let a = Matrix::new([1, 2, 3, 4], 2, 2);
        let b = Matrix::new([1, 2, 3, 4], 2, 2);
        let c = a * b;
        assert_eq!(c.data, vec![7, 10, 15, 22]);
        assert_eq!(format!("{}", c), "{7 10, 15 22}");

        Ok(())
    }

    #[test]
    fn test_a_can_not_multiply_b() {
        let a = Matrix::new([1, 2, 3, 4, 5, 6], 2, 3);
        let b = Matrix::new([1, 2, 3, 4, 5, 6], 2, 2);
        let c = multiply(&a, &b);
        assert!(c.is_err());
    }

    #[test]
    #[should_panic]
    fn test_a_can_not_multiply_b_panic() {
        let a = Matrix::new([1, 2, 3, 4, 5, 6], 2, 3);
        let b = Matrix::new([1, 2, 3, 4, 5, 6], 2, 2);
        let _c = a * b;
    }
}
