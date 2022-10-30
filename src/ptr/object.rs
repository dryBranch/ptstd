/// 这是一个简陋的裸指针包装
/// ## 目的
///     - 解决麻烦的引用和生命周期语法而使用裸指针
///     - 裸指针解引用语法可能很难看
/// 
/// ## 注
///     - 隐藏了 unsafe 所以很危险
///     - 使用 new 和 free 会与 CPP 一样会造成内存泄漏和 double free
///     
/// 

use std::{ops::{Deref, DerefMut}, fmt::Display};
use std::ptr::null_mut;

/// 对裸指针的包装
#[derive(Copy, Debug)]
pub struct Object<T>(*mut T);

/// 自动解引用
impl<T> Deref for Object<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0 }
    }
}

impl<T> DerefMut for Object<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.0 }
    }
}

/// 传递可打印的特征
impl<D: Display> Display for Object<D> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_mut())
    }
}

/// 解决有些无法实现自动 Clone 的问题
impl<T> Clone for Object<T> {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl<T> Object<T> {
    /// 破坏性地将不可变转为可变
    pub fn to_mut(&self) -> &mut T {
        unsafe { &mut *self.0 }
    }

    /// 获得该指针指向的对象并复制到栈上
    /// 将会释放堆上的对象
    pub fn get(self) -> Option<T> {
        if !self.0.is_null() {
            let b = unsafe { Box::from_raw(self.0) };
            return Some( *b );
        }
        None
    }

    /// 指针复制
    pub fn duplicate(&self) -> Self {
        Self(self.0)
    }

    /// 如果非空执行
    pub fn ok_then<F>(&self, f: F)
        where F: FnOnce(Self)
    {
        if !self.0.is_null() {
            f(self.clone())
        }
    }
}

// ================= 工具函数 =====================

/// 空指针
pub fn null<T>() -> Object<T> {
    Object(null_mut())
}

/// 在堆上创建一个对象并转为对象指针
pub fn new<T>(o: T) -> Object<T> {
    let p = Box::leak(
        Box::new(o)
    );
    Object(p)
}

/// 释放堆上的指针
pub fn free<T>(o: Object<T>) {
    unsafe {Box::from_raw(o.0)};
}

/// 将一个已有的引用转为对象指针
pub fn from_ref<T>(o: &T) -> Object<T> {
    let p = o as *const T as *mut T;
    Object(p)
}