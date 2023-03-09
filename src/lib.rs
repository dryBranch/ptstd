pub mod net;
pub mod ptr;
pub mod crypto;
pub mod thread;
pub mod linear;

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::*;
    
    #[test]
    fn test1() {
        let p = Rc::new("1234");
        println!("{:?}", p);
    }
}