#![crate_type = "dylib"]
#![allow(unused_variables)]
//TODO 
//This is a c style include. It copyies the entire rust lib file into this compilation unit.
//We should attempt to clean this up some time to make it more rust like. For the moment however
//this seem to be the most straight forward way to interact with the basic interface lib 

include!("../src/lib.rs");

//Instructions for how to compile 
//rustc test.rs
//if attribute macro found at the top of this file is not present compile with the following
//rustc test.rs --crate-type=dylib 


use render_tools::*;
use memory_tools::*;
use interaction_tools::*;


#[no_mangle]
fn UD_test(ri: &mut RenderInstructions, gs: &mut GlobalStorage, ls: &mut LocalStorage, inputs: &InteractiveInfo)->Result<(), String>{
    ls.interactive = true;
    //ri.println("Hello World");
    ri.clear();
    ri.draw_rect([0.0 + inputs.frames%100 as f32 * 0.1, 0.0, 0.4, 0.4], [1.0;4], true);
    return Ok(());
}


#[no_mangle]
fn UD_test1(ri: &mut RenderInstructions, gs: &mut GlobalStorage, ls: &mut LocalStorage, inputs: &InteractiveInfo)->Result<(), String>{
    gs.store([1.0f32; 5], "stuff");
    gs.store(Abc{a: 10.0}, "stuff2");
    ri.println("done!");
    return Ok(());
}

#[derive(Debug)]
struct Abc{
    a : f32,
}

#[no_mangle]
fn UD_test2(ri: &mut RenderInstructions, gs: &mut GlobalStorage, ls: &mut LocalStorage, inputs: &InteractiveInfo)->Result<(), String>{
    let l : [f32; 5] = *gs.get("stuff")?;
    ri.println(&format!("{:?}", l));
    let abc : &Abc = gs.get("stuff2")?;
    ri.println(&format!("{:?}", abc));
    return Ok(());
}
