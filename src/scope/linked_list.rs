// Shamelessly copied and adapted from: https://github.com/rust-unofficial/too-many-lists/blob/master/lists/src/third.rs (16.12.2016)
// (c) Alexis Beingessner

use std::rc::Rc;

#[derive(Debug, PartialEq)]
pub struct List<T> {
    list_head: Link<T>,
}

type Link<T> = Option<Rc<Node<T>>>;

#[derive(Debug, Clone, PartialEq)]
struct Node<T> {
    elem: T,
    next: Link<T>,
}

impl<T> List<T> {
    pub fn new() -> Self {
        List { list_head: None }
    }

    pub fn append(&self, elem: T) -> List<T> {
        List {
            list_head: Some(
                    Rc::new(Node {
                        elem: elem,
                        next: self.list_head.clone(),
                    })
            )
        }
    }

    /*
    pub fn tail(&self) -> List<T> {
        List { list_head: self.list_head.as_ref().and_then(|node| node.next.clone()) }
    }
    */

    pub fn head(&self) -> Option<&T> {
        self.list_head.as_ref().map(|node| &node.elem)
    }

    pub fn iter(&self) -> Iter<T> {
        Iter::new(self)
    }
}

// This clones the first element of the list and returns a "new" list
// with the cloned head as head
impl<T> Clone for List<T> {
    fn clone(&self) -> Self {
        List { list_head: self.list_head.clone() }
    }
}

#[derive(Debug)]
pub struct Iter<'a, T:'a> {
    next: Option<&'a Node<T>>,
}

impl<'a, T> Iter<'a, T> {
    fn new(list: &'a List<T>) -> Iter<'a, T> {
        Iter {
            next: list.list_head.as_ref().map(|node| &**node)
        }
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.next.map(|node| {
            self.next = node.next.as_ref().map(|node| &**node);
            &node.elem
        })
    }
}
