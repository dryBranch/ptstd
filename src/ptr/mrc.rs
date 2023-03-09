use std::{rc::{Rc, Weak}, ops::{Deref, DerefMut}, fmt::{Display, Debug}, hash::Hash};

/// 对内部对象的包装
struct Pointer<T: ?Sized>(*mut T);

impl<T: ?Sized> Deref for Pointer<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0 }
    }
}

// impl<T: ?Sized> DerefMut for Pointer<T> {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         unsafe { &mut *self.0 }
//     }
// }

impl<T: ?Sized> Drop for Pointer<T> {
    fn drop(&mut self) {
        unsafe { Box::from_raw(self.0) };
    }
}

impl<T: ?Sized + Display> Display for Pointer<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self.deref(), f)
    }
}

impl<T: ?Sized + Debug> Debug for Pointer<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self.deref(), f)
    }
}

impl<T: ?Sized + PartialEq> PartialEq for Pointer<T> {
    fn eq(&self, other: &Self) -> bool {
        self.deref().eq(other)
    }
}

impl<T: ?Sized + Eq> Eq for Pointer<T> { }

impl<T: ?Sized + PartialOrd> PartialOrd for Pointer<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.deref().partial_cmp(other)
    }
}

impl<T: ?Sized + Ord> Ord for Pointer<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.deref().cmp(other)
    }
}

impl<T: ?Sized + Hash> Hash for Pointer<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.deref().hash(state)
    }
}

/// 多重所有权可变引用
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Mrc<T: ?Sized>(Rc<Pointer<T>>);

impl<T> Deref for Mrc<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

/// 使其可变
impl<T> DerefMut for Mrc<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.0.0 }
    }
}

impl<T> Clone for Mrc<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: ?Sized + Display> Display for Mrc<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<T: ?Sized + Debug> Debug for Mrc<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt((*self.0).deref(), f)
    }
}


impl<T> Mrc<T> {
    pub fn new(value: T) -> Self {
        Self(Rc::new(
            Pointer(Box::into_raw(Box::new(
                value
            )))
        ))
    }

    /// 降级获得相应的弱引用
    pub fn downgrade(this: &Self) -> Mweak<T> {
        Mweak(Rc::downgrade(&this.0))
    }

    /// 当期只有一个强引用时解包
    /// 失败则原路返回
    pub fn try_unwrap(self) -> Result<T, Mrc<T>> {
        Rc::try_unwrap(self.0)
            .map(|p| unsafe { *Box::from_raw(p.0) } )
            .map_err(|p| Mrc(p))
    }

    /// 得到可变引用
    pub unsafe fn to_mut(&self) -> &mut T {
        &mut *self.0.0
    }
}

impl<T: Clone> Mrc<T> {
    pub fn unwrap_or_clone(self) -> T {
        Mrc::try_unwrap(self)
            .unwrap_or_else(|rc| (*rc).clone() )
    }
}

/// 对应的弱引用
pub struct Mweak<T: ?Sized>(Weak<Pointer<T>>);

impl<T> Mweak<T> {
    pub fn upgrade(&self) -> Option<Mrc<T>> {
        Weak::upgrade(&self.0)
            .map(|p| Mrc(p))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test1() {
        let mut a = Mrc::new(1);
        let mut b = a.clone();
        println!("{} {}", a, b);
        *b += 1;
        *a += 2;
        println!("{} {}", a, b);
        println!("{:?} {:?}", a, b);
        let a = Rc::new(1);
        println!("{a}, {a:?}");

        let s = Rc::new("rc");
        println!("{}, {:?}", s, s);
        let s = Mrc::new("mrc");
        println!("{}, {:?}", s, s);
    }

    #[test]
    fn test2() {
        let a = Mrc::new("hello".to_string());
        let b = a.clone();
        // assert!(a.try_unwrap().is_err());
        // assert!(b.try_unwrap().is_ok());
        println!("{}", Rc::strong_count(&a.0));
        println!("{}", Rc::strong_count(&b.0));
        println!("{:?}, {}", a.try_unwrap(), Rc::strong_count(&b.0));
        
        println!("{}", Rc::strong_count(&b.0));
        println!("{:?}", b.try_unwrap());
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

    #[test]
    fn test_ord() {
        let a = Mrc::new(1);
        let b = a.clone();
        let mut c = Mrc::new(1);
        println!("{}", a == b);
        println!("{}", a == c);
        *c += 1;
        println!("{}", b < c);

        let a = Rc::new(1);
        let c = Rc::new(1);
        println!("{}", a == c);
    }

    #[test]
    fn test_box() {
        let a = Box::into_raw(Box::new(
            Person {
                name    : "tom".to_string(),
                id      : 1,
            }
        ));
        unsafe {
            let mut b = Box::from_raw(a);
            let mut c = *b;
            c.id = 10;
        }
        println!("end");
    }
}