#![feature(plugin)]
#![plugin(clippy)]

#![deny(len_without_is_empty, len_zero)]
#![allow(dead_code, unused)]

pub struct PubOne;

impl PubOne {
    pub fn len(self: &Self) -> isize { //~ERROR item `PubOne` has a public `len` method but no corresponding `is_empty`
        1
    }
}

struct NotPubOne;

impl NotPubOne {
    pub fn len(self: &Self) -> isize { // no error, len is pub but `NotPubOne` is not exported anyway
        1
    }
}

struct One;

impl One {
    fn len(self: &Self) -> isize { // no error, len is private, see #1085
        1
    }
}

pub trait PubTraitsToo {
    fn len(self: &Self) -> isize; //~ERROR trait `PubTraitsToo` has a `len` method but no `is_empty`
}

impl PubTraitsToo for One {
    fn len(self: &Self) -> isize {
        0
    }
}

trait TraitsToo {
    fn len(self: &Self) -> isize; // no error, len is private, see #1085
}

impl TraitsToo for One {
    fn len(self: &Self) -> isize {
        0
    }
}

struct HasPrivateIsEmpty;

impl HasPrivateIsEmpty {
    pub fn len(self: &Self) -> isize {
        1
    }

    fn is_empty(self: &Self) -> bool {
        false
    }
}

pub struct HasIsEmpty;

impl HasIsEmpty {
    pub fn len(self: &Self) -> isize { //~ERROR item `HasIsEmpty` has a public `len` method but a private `is_empty`
        1
    }

    fn is_empty(self: &Self) -> bool {
        false
    }
}

struct Wither;

pub trait WithIsEmpty {
    fn len(self: &Self) -> isize;
    fn is_empty(self: &Self) -> bool;
}

impl WithIsEmpty for Wither {
    fn len(self: &Self) -> isize {
        1
    }

    fn is_empty(self: &Self) -> bool {
        false
    }
}

pub struct HasWrongIsEmpty;

impl HasWrongIsEmpty {
    pub fn len(self: &Self) -> isize { //~ERROR item `HasWrongIsEmpty` has a public `len` method but no corresponding `is_empty`
        1
    }

    pub fn is_empty(self: &Self, x : u32) -> bool {
        false
    }
}

fn main() {
    let x = [1, 2];
    if x.len() == 0 {
        //~^ERROR length comparison to zero
        //~|HELP consider using `is_empty`
        //~|SUGGESTION x.is_empty()
        println!("This should not happen!");
    }

    if "".len() == 0 {
        //~^ERROR length comparison to zero
        //~|HELP consider using `is_empty`
        //~|SUGGESTION "".is_empty()
    }

    let y = One;
    if y.len()  == 0 { //no error because One does not have .is_empty()
        println!("This should not happen either!");
    }

    let z : &TraitsToo = &y;
    if z.len() > 0 { //no error, because TraitsToo has no .is_empty() method
        println!("Nor should this!");
    }

    let has_is_empty = HasIsEmpty;
    if has_is_empty.len() == 0 {
        //~^ERROR length comparison to zero
        //~|HELP consider using `is_empty`
        //~|SUGGESTION has_is_empty.is_empty()
        println!("Or this!");
    }
    if has_is_empty.len() != 0 {
        //~^ERROR length comparison to zero
        //~|HELP consider using `is_empty`
        //~|SUGGESTION !has_is_empty.is_empty()
        println!("Or this!");
    }
    if has_is_empty.len() > 0 {
        //~^ERROR length comparison to zero
        //~|HELP consider using `is_empty`
        //~|SUGGESTION !has_is_empty.is_empty()
        println!("Or this!");
    }
    assert!(!has_is_empty.is_empty());

    let with_is_empty: &WithIsEmpty = &Wither;
    if with_is_empty.len() == 0 {
        //~^ERROR length comparison to zero
        //~|HELP consider using `is_empty`
        //~|SUGGESTION with_is_empty.is_empty()
        println!("Or this!");
    }
    assert!(!with_is_empty.is_empty());

    let has_wrong_is_empty = HasWrongIsEmpty;
    if has_wrong_is_empty.len() == 0 { //no error as HasWrongIsEmpty does not have .is_empty()
        println!("Or this!");
    }
}
