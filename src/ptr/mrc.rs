use std::{rc::Rc, ops::{Deref, DerefMut}, fmt::{Display, Debug}};

/// 多重所有权可变引用
#[derive(Debug)]
pub struct Mrc<T: ?Sized>(Rc<T>);

impl<T> Deref for Mrc<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

/// 使其可变
impl<T> DerefMut for Mrc<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // 危险的黑魔法
        unsafe { std::mem::transmute(self.0.deref() as *const Self::Target) }
    }
}

impl<T> Clone for Mrc<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: Display> Display for Mrc<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}


impl<T> Mrc<T> {
    pub fn new(value: T) -> Self {
        Self(Rc::new(value))
    }

    /// 当期只有一个强引用时解包
    /// 失败则原路返回
    pub fn try_unwrap(self) -> Result<T, Mrc<T>> {
        Rc::try_unwrap(self.0)
            .map_err(|p| Mrc(p))
    }

    /// 得到可变引用
    pub unsafe fn to_mut(&self) -> &mut T {
        std::mem::transmute(self.0.deref() as *const T)
    }
}

impl<T: Clone> Mrc<T> {
    pub fn unwrap_or_clone(self) -> T {
        Rc::try_unwrap(self.0).unwrap_or_else(|rc| (*rc).clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test1() {
        let a = Mrc::new(1);
        let mut b = a.clone();
        println!("{} {}", a, b);
        *b += 1;
        println!("{} {}", a, b);
        println!("{:?} {:?}", a, b);
    }

    #[test]
    fn test2() {
        let a = Mrc::new("hello".to_string());
        let b = a.clone();
        assert!(a.try_unwrap().is_err());
        assert!(b.try_unwrap().is_ok());
    }

    struct Person {
        name    : String,
        id      : u32,
    }
    
    impl Drop for Person {
        fn drop(&mut self) {
            println!("Person droped name: {}, id: {}", self.name, self.id);
        }
    }

    #[test]
    fn test_drop() {
        let p1 = Person {
            name    : "tom".to_string(),
            id      : 1,
        };
        {
            let a = Mrc::new(p1);
            let mut b = a.clone();
            b.id += 2;
        }
        println!("end");
    }
}