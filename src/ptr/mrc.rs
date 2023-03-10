use std::{rc::{Rc, Weak}, ops::{Deref, DerefMut}, fmt::{Display, Debug}, hash::Hash};

/// 对内部对象的包装
struct Pointer<T: ?Sized>(*mut T, bool);

impl<T: ?Sized> Deref for Pointer<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0 }
    }
}

impl<T: ?Sized> Drop for Pointer<T> {
    #[inline]
    fn drop(&mut self) {
        if self.1 {
            unsafe { Box::from_raw(self.0) };
        }
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
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.deref().eq(other)
    }
}

impl<T: ?Sized + Eq> Eq for Pointer<T> { }

impl<T: ?Sized + PartialOrd> PartialOrd for Pointer<T> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.deref().partial_cmp(other)
    }
}

impl<T: ?Sized + Ord> Ord for Pointer<T> {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.deref().cmp(other)
    }
}

impl<T: ?Sized + Hash> Hash for Pointer<T> {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.deref().hash(state)
    }
}

/// 多重所有权可变引用
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Mrc<T: ?Sized>(Rc<Pointer<T>>);

impl<T> Deref for Mrc<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

/// 使其可变
impl<T> DerefMut for Mrc<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.0.0 }
    }
}

impl<T> Clone for Mrc<T> {
    #[inline]
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

// 既然要追求刺激，那就贯彻到底咯
unsafe impl<T> Sync for Mrc<T> { }
unsafe impl<T> Send for Mrc<T> { }

impl<T> Mrc<T> {
    pub fn new(value: T) -> Self {
        Self(Rc::new(
            Pointer(
                Box::into_raw(Box::new(value)), 
                true,
            )
        ))
    }

    /// 降级获得相应的弱引用
    #[inline]
    pub fn downgrade(this: &Self) -> Mweak<T> {
        Mweak(Rc::downgrade(&this.0))
    }

    /// 得到强引用数量
    #[inline]
    pub fn strong_count(this: &Self) -> usize {
        Rc::strong_count(&this.0)
    }

    /// 得到若引用数量
    #[inline]
    pub fn weak_count(this: &Self) -> usize {
        Rc::weak_count(&this.0)
    }

    /// 当期只有一个强引用时解包
    /// 失败则原路返回
    pub fn try_unwrap(self) -> Result<T, Mrc<T>> {
        Rc::try_unwrap(self.0)
            .map(|mut p| unsafe { 
                // 阻止回收堆上内存
                p.1 = false;
                *Box::from_raw(p.0)
            } )
            .map_err(|p| Mrc(p))
    }

    /// 得到可变引用
    #[inline]
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
    fn test_mut() {
        let mut a = Mrc::new(1);
        let mut b = a.clone();
        *b += 1;
        assert!(*a == 2);
        *a += 2;
        assert!(*b == 4);
        assert!(*a == *b);
    }

    #[test]
    fn test_strong_count() {
        let a = Mrc::new("hello".to_string());
        assert!(Mrc::strong_count(&a) == 1);
        let b = a.clone();
        assert!(Mrc::strong_count(&a) == 2);
        assert!(Mrc::strong_count(&b) == 2);
        
        assert!(a.try_unwrap().is_err());
        assert!(Mrc::strong_count(&b) == 1);
        assert!(b.try_unwrap().is_ok());
    }

    #[derive(Debug)]
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
            assert!(a.try_unwrap().is_err());
            assert!(b.try_unwrap().is_ok());
        }
        println!("end");
    }

    #[test]
    fn test_ord() {
        let a = Mrc::new(1);
        let b = a.clone();
        let mut c = Mrc::new(1);
        assert!(a == b);
        assert!(a == c);
        *c += 1;
        assert!(b < c);
        assert!(c > a);
    }
}