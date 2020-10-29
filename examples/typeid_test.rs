#![allow(dead_code)]

use std::any::TypeId;

struct A(f32);
struct Abc{
    a: A,
}

fn main(){
    

    println!("{:?}", TypeId::of::<Abc>())
}
