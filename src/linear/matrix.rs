/// 给二维数组使用的宏
/// ---
/// 用法与 `Python` 中一致
/// ## 注
/// 为了方便使用，会 `clone` 第一个矩阵
/// ```
/// let a = array![
///     [1, 2],
///     [3, 4]
/// ];
/// let b = a.clone();
/// let c = matrix_mul!(a @ b); // a & b is avilable
/// ```
#[macro_export]
macro_rules! matrix_mul {
    ( $a: ident @ $($b: ident)@* ) => {
        $a.clone()
            $(.dot(&$b))*
    };
}

/// 方便创建矩阵，减少写括号
/// ```
/// let a = matrix![
///     1, 2;
///     3, 4
/// ];
/// ```
#[macro_export]
macro_rules! matrix {
    ( $($($x:expr),+ $(,)?);* $(;)? ) => (
        {
            use ndarray::array;
            array![$(
                [$($x),+],
            )*]
        }
    );
}

#[cfg(test)]
mod tests {
    use ndarray::array;
    #[test]
    fn test1() {
        let a = matrix![
            1, 2;
            3, 4
        ];

        let b = array![
            [1, 2],
            [3, 4],
        ];

        let c = matrix_mul!(a @ b @ b);
        println!("{}", c);
        println!("{}", a);
        println!("{}", b);
    }
}