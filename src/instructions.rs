//TODO 
//name is nolonger acc. plz change
//Organizationally this is probably bad for cache
extern crate glium;

use lib::dynamic_lib_loading;
use lib::dynamic_lib_loading::{open_lib, get_fn, get_error, close_lib, DyLib};
use lib::memory_tools::{GlobalStorage, LocalStorage};
use lib::interaction_tools::{InteractiveInfo};
use lib::render_tools::{Bitmap, RenderInstructions};

use glium::texture::Texture2d;
use Vertex;
use UserFn;

struct InstructionBuffer{
    bitmaps      : Vec<Bitmap>,
    textures     : Vec<Texture2d>,
    vertexbuffers: Vec<glium::VertexBuffer<Vertex>>,
    fns          : Vec<UserFn>,
    fns_source   : Vec<String>,
    translations : Vec<[f32; 2]>,

    render_instructions: Vec<RenderInstructions>,
    localstorage       : Vec<LocalStorage>,
    interactiveinputs  : Vec<InteractiveInfo>,  //TODO we could but prob should not use InteractiveInfo infocus to help with moving and expanding

    ids               : Vec<String>,
    bitmap_is_uptodate: Vec<bool>, //i don't like this name

    machine_edit_canvas_mode: Vec<bool>, //i don't like this name

    src_path: String,
}

impl InstructionBuffer{
    fn new()->InstructionBuffer{
        InstructionBuffer{  bitmaps      : Vec::new(), 
                            textures     : Vec::new(), 
                            vertexbuffers: Vec::new(), 
                            fns          : Vec::new(), 
                            fns_source   : Vec::new(), 
                            translations : Vec::new(), 
                            render_instructions: Vec::new(),
                            localstorage       : Vec::new(),
                            ids                : Vec::new(),
                            bitmap_is_uptodate : Vec::new(),      //I don't like this name
                            interactiveinputs  : Vec::new(),
                            machine_edit_canvas_mode: Vec::new(), //i don't like this name
                            src_path: String::new(),
        }
    }
    fn push( &mut self, bmp: Bitmap, vertexbuffer: glium::VertexBuffer<Vertex>, texture: Texture2d, id: String, source: String,
             func: UserFn ){
        self.bitmaps.push(bmp); 
        self.textures.push(texture); 
        self.vertexbuffers.push(vertexbuffer); 
        self.fns.push(func); 
        self.fns_source.push(source); 

        let l = self.bitmaps.len();
        let x = -1.0 * (l%2) as f32 + 0.2 * l as f32;
        let y = -1.0 + 0.1*(l/2) as f32;
        self.translations.push([x, y]);

        self.render_instructions.push( Default::default() );
        self.localstorage.push(LocalStorage::new());
        self.ids.push(id); 
        self.bitmap_is_uptodate.push(false);  //i don't like this name
        self.interactiveinputs.push(Default::default());
        self.machine_edit_canvas_mode.push(false); //i don't like this name

        //CLEANUP
        // this is kinda stupid
        // there might be a more elagante way of doing this.
        if  self.bitmaps.len() != self.ids.len() {panic!("InstructionBuffer bitmaps {} and ids {} do not agree", self.bitmaps.len(), self.ids.len());} 
        if  self.bitmaps.len() != self.fns.len(){panic!("InstructionBuffer bitmaps and fns do not agree");} 
        if  self.bitmaps.len() != self.vertexbuffers.len() {panic!("InstructionBuffer bitmaps and vrtxbufferss do not agree");}
        if  self.bitmaps.len() != self.render_instructions.len(){panic!(format!("InstructionBuffer bitmaps {} and renderinstructions do not agree {}", self.bitmaps.len(), self.render_instructions.len()));} 
        if  self.bitmap_is_uptodate.len() != self.bitmaps.len(){panic!("InstructionBuffer bitmaps and bmp_uptodate do not agree");} 
        if  self.localstorage.len() != self.bitmaps.len(){panic!("InstructionBuffer bitmaps and localstorage do not agree");} 
        if  self.localstorage.len() != self.fns_source.len(){panic!("InstructionBuffer fns_source and localstorage do not agree");} 
        if  self.localstorage.len() != self.interactiveinputs.len() 
           { panic!("InstructionBuffer lengths localstorage and interactiveinputs are different."); }
    }
    fn len(&self)->usize{
        return self.bitmaps.len();
    }
    fn contains(&self, id: &str)->bool{
        for it in self.ids.iter(){
            if it == id{ return true;}
        }
        return false;
    }
    fn get_index(&self, id: &str)->Result<usize, String>{
        for (i, it) in self.ids.iter().enumerate(){
            if it == id{ return Ok(i);}
        }
        return Err(format!("Id '{}' not found", id));
    }
}


