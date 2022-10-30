use ptstd::ptr::mrc::Mrc;

type Link<T> = Option<Mrc<Node<T>>>;

struct List<T> {
    head    : Link<T>,
    tail    : Link<T>,
    length  : usize,
}

struct Node<T> {
    prev    : Link<T>,
    next    : Link<T>,
    value   : T,
}

impl<T> Node<T> {
    fn new(v: T) -> Mrc<Node<T>> {
        Mrc::new(Node {
            prev    : None,
            next    : None,
            value   : v,
        })
    }
}

impl<T> Drop for List<T> {
    fn drop(&mut self) {
        println!("list drop");
        while let Some(_) = self.pop_back() {

        }
    }
}

impl<T> List<T> {
    fn new() -> List<T> {
        List {
            head    : None,
            tail    : None,
            length  : 0,
        }
    }

    fn push_back(&mut self, v: T) {
        let mut e = Node::new(v);
        match self.tail.take() {
            Some(mut old_tail) => {
                old_tail.next = Some(e.clone());
                e.prev = Some(old_tail);
            },
            None => {
                self.head = Some(e.clone());
            },
        }
        self.tail = Some(e);
        self.length += 1;
    }

    fn push_front(&mut self, v: T) {
        let mut e = Node::new(v);
        match self.head.take() {
            Some(mut old_head) => {
                old_head.prev = Some(e.clone());
                e.next = Some(old_head);
            },
            None => {
                self.tail = Some(e.clone());
            },
        }
        self.head = Some(e);
        self.length += 1;
    }

    fn pop_back(&mut self) -> Option<T> {
        self.tail.take().map(|mut old_tail| {
            match old_tail.prev.take() {
                Some(mut new_tail) => {
                    new_tail.next = None;
                    self.tail = Some(new_tail);
                },
                None => {
                    self.head = None;
                },
            };
            self.length -= 1;
            old_tail.try_unwrap()
                .ok()
                .unwrap()
                .value
        })
    }

    fn pop_front(&mut self) -> Option<T> {
        self.head.take().map(|mut old_head| {
            match old_head.next.take() {
                Some(mut new_head) => {
                    new_head.prev = None;
                    self.head = Some(new_head);
                },
                None => {
                    self.tail = None;
                },
            };
            self.length -= 1;
            old_head.try_unwrap()
                .ok()
                .unwrap()
                .value
        })
    }
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
fn test_function() {
    let mut l = List::new();
    for i in 1..=10 {
        l.push_back(i);
    }

    for _ in 1..=11 {
        let e = l.pop_back();
        println!("{:?}, left = {}", e, l.length);
    }
}

#[test]
fn test_drop() {
    let mut l = List::new();

    for i in 1..=10 {
        let p = Person {
            name    : format!("person{}", i),
            id      : i + 1,
        };
        l.push_front(p);
    }
    for _ in 1..=10 {
        let e = l.pop_front();
        println!("{}, left = {}", e.unwrap().name, l.length);
    }
    println!("{}", l.length);
}