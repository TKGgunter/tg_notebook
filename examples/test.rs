#![crate_type = "dylib"]
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
    ri.println("Testing Testing 1,2,3");
    ri.println("Ready for blast off?");
    ri.println("All systems check!");
    ri.println("Blastoff!!!!");
    return Ok(());
}

#[no_mangle]
fn UD_main(ri: &mut RenderInstructions, gs: &mut GlobalStorage, ls: &mut LocalStorage, inputs: &InteractiveInfo)->Result<(), String>{
    ri.println("Hello World!");
        //pub fn store<T: 'static>(&mut self, v: T, name: &str)->Result<(), String>{
    gs.store([1.0f32;4], "some_color");
    gs.get::<[f32;4]>("some_color")?;
    
    return Ok(());
}


