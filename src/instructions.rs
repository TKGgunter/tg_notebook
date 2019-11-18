//TODO 
//name is nolonger acc. plz change
//Organizationally this is probably bad for cache
extern crate glium;

use lib::dynamic_lib_loading;
use lib::dynamic_lib_loading::{open_lib, get_fn, get_error, close_lib, DyLib};
use lib::memory_tools::{GlobalStorage, LocalStorage};
use lib::interaction_tools::{InteractiveInfo};
use lib::render_tools::{RenderInstructions, BitmapContainer};

use glium::texture::Texture2d;
use UserFn;

pub struct InstructionBuffer{
    pub initialized  : Vec<bool>,

    pub bitmaps      : Vec<BitmapContainer>,
    pub textures     : Vec<Texture2d>,
    pub fns          : Vec<UserFn>,
    pub fns_source   : Vec<String>,
    pub pos_rect     : Vec<[f32; 4]>,
    pub println_y    : Vec<f32>,
    pub max_println_y    : Vec<f32>,

    pub render_instructions: Vec<RenderInstructions>,
    pub localstorage       : Vec<LocalStorage>,
    pub interactive        : Vec<bool>,  //TODO we could but prob should not use InteractiveInfo infocus to help with moving and expanding
    pub infocus            : Vec<bool>,  

    pub ids               : Vec<String>,

    pub machine_edit_canvas_mode: Vec<bool>, //i don't like this name

    pub src_path: String,
}

impl InstructionBuffer{
    pub fn new()->InstructionBuffer{
        InstructionBuffer{  initialized  : Vec::new(),
                            bitmaps      : Vec::new(), 
                            textures     : Vec::new(), 
                            fns          : Vec::new(), 
                            fns_source   : Vec::new(), 
                            pos_rect     : Vec::new(), 
                            println_y    : Vec::new(),
                            max_println_y    : Vec::new(),
                            render_instructions: Vec::new(),
                            localstorage       : Vec::new(),
                            ids                : Vec::new(),
                            interactive        : Vec::new(),
                            infocus            : Vec::new(),
                            machine_edit_canvas_mode: Vec::new(), //i don't like this name
                            src_path: String::new(),
        }
    }
    pub fn push( &mut self, bmp: BitmapContainer, texture: Texture2d, id: String, source: String,
             func: UserFn ){
        self.initialized.push(false); 

        self.bitmaps.push(bmp); 
        self.textures.push(texture); 
        self.fns.push(func); 
        self.fns_source.push(source); 

        let l = self.bitmaps.len();
        self.pos_rect.push([0.0;4]);
        self.println_y.push(-0.0);
        self.max_println_y.push(-0.0);

        self.render_instructions.push( Default::default() );
        self.localstorage.push(LocalStorage::new());
        self.ids.push(id); 
        self.interactive.push(false);
        self.infocus.push(false);
        self.machine_edit_canvas_mode.push(false); //i don't like this name

        //CLEANUP
        // this is kinda stupid
        // there might be a more elagante way of doing this.
        if  self.bitmaps.len() != self.ids.len() {panic!("InstructionBuffer bitmaps {} and ids {} do not agree", self.bitmaps.len(), self.ids.len());} 
        if  self.bitmaps.len() != self.fns.len(){panic!("InstructionBuffer bitmaps and fns do not agree");} 
        if  self.bitmaps.len() != self.render_instructions.len(){panic!(format!("InstructionBuffer bitmaps {} and renderinstructions do not agree {}", self.bitmaps.len(), self.render_instructions.len()));} 
        if  self.localstorage.len() != self.bitmaps.len(){panic!("InstructionBuffer bitmaps and localstorage do not agree");} 
        if  self.localstorage.len() != self.fns_source.len(){panic!("InstructionBuffer fns_source and localstorage do not agree");} 
        if  self.localstorage.len() != self.interactive.len() 
           { panic!("InstructionBuffer lengths localstorage and interactiveinputs are different."); }
    }
    pub fn len(&self)->usize{
        return self.bitmaps.len();
    }
    pub fn contains(&self, id: &str)->bool{
        for it in self.ids.iter(){
            if it == id{ return true;}
        }
        return false;
    }
    pub fn get_index(&self, id: &str)->Result<usize, String>{
        for (i, it) in self.ids.iter().enumerate(){
            if it == id{ return Ok(i);}
        }
        return Err(format!("Id '{}' not found", id));
    }
}


