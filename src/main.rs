#![allow(dead_code)]

#[macro_use]
extern crate glium;
extern crate nalgebra; //TODO we can remove this dependancy if we want to we don't use it right now
extern crate stb_tt_sys;

use nalgebra::core::Matrix4; //TODO this can be removed if we want
use glium::{glutin, Surface};

use stb_tt_sys::*;
use std::ptr::{null_mut};

mod lib;
use lib::dynamic_lib_loading;
use lib::dynamic_lib_loading::{open_lib, get_fn, close_lib, DyLib};
use lib::memory_tools::{GlobalStorage, LocalSettingsStorage};
use lib::interaction_tools::{InteractiveInfo};
use lib::render_tools::{RenderInstructions, RenderType};

mod instructions;
use instructions::InstructionBuffer;

//NOTE
//what is the time cost of loading various resolutions of textures all the time : it takes about 2 millis to generate a high res bmp
//To generate a texture and push that texture to the gpu it takes another 2-3 millisecs

//TODO
// + scroll for panels
// + fix alpha blending
// + interactivity
// + char map will run out if space. we should handle this


//BUGS
// + When render symbols are on different lines in texture alignment get out of sync


const C_BLACK : [f32; 4] = [0.0, 0.0, 0.0, 1.0];
const C_WHITE : [f32; 4] = [1.0, 1.0, 1.0, 1.0];
const C_RED   : [f32; 4] = [1.0, 0.0, 0.0, 1.0];
const C_GREEN : [f32; 4] = [0.0, 1.0, 0.0, 1.0];
const C_BLUE  : [f32; 4] = [0.0, 0.0, 1.0, 1.0];
const C_GREY  : [f32; 4] = [0.5, 0.5, 0.5, 1.0];
const C_DARKGREY  : [f32; 4] = [0.15, 0.15, 0.15, 1.0];
const C_LIGHTBLACK : [f32; 4] = [0.025, 0.025, 0.025, 1.0];
const C_PURPLE: [f32; 4] = [0.5, 0.0, 0.5, 1.0];

const DEFAULT_TEXTURE_WIDTH_FRAC  : f32 = 1.7;
const DEFAULT_TEXTURE_HEIGHT_FRAC : f32 = 0.45;


type UserFn = fn(&mut RenderInstructions, &mut GlobalStorage, &mut LocalSettingsStorage, &InteractiveInfo)->Result<(), String>;






static mut DEFAULT_FONT_BUFFER : &'static [u8] = include_bytes!("../assets/RobotoMono-Regular.ttf");








#[derive(Clone, Copy)]
pub struct WindowInfo{
    pub focused: bool,
    pub width  : f64,
    pub height : f64,
}


static mut WINDOWINFO : WindowInfo = WindowInfo{  focused: false,
                                                  width  : 0.0,
                                                  height : 0.0,
                                                };


fn clone_windowinfo()->WindowInfo{ unsafe{
    WINDOWINFO.clone()
}}

fn set_windoinfo(winfo: WindowInfo){unsafe{
    WINDOWINFO = winfo; 
}}













pub static mut GLOBAL_FONTINFO : stbtt_fontinfo = new_stbtt_fontinfo();

pub fn initfont(font_buffer: &[u8]){unsafe{
    if stbtt_InitFont(&mut GLOBAL_FONTINFO as *mut stbtt_fontinfo, font_buffer.as_ptr(), 0) == 0{
        panic!("font was not able to load.");
    }
}}



type FontLib<'a> = std::collections::HashMap<String, &'a[u8]>; //TODO TinyString

#[derive(Eq, PartialEq, Hash, Clone)]
struct CharKey{
    symbol:    char,
    size:      u32,
    font_name: Option<String>,//TODO TinyString,
}

struct CharMap{
    texture: glium::texture::Texture2d,
    map:     std::collections::HashMap<CharKey, [f32; 4]>,
    cursor: [f32; 2],
    current_maxdepth: f32,
}
impl CharMap{
    pub fn new(display: &glium::Display)->CharMap{
        CharMap{
            texture: glium::texture::Texture2d::empty(display, 1000, 1000)
                                .expect("Texture could not be generated."),
            map    : std::collections::HashMap::new(),
            cursor: [-1.0, -1.0],
            current_maxdepth: 0.0,
            
        }
    }
    pub fn insert(&mut self, display: &glium::Display, program: &glium::Program, indices: &glium::index::NoIndices, charkey: CharKey, fontlib: &FontLib){
    //TODO
    //might need to change how we render to texture

        if self.map.contains_key(&charkey){
            return;
        }
        let _charkey = charkey.clone();
        let CharKey{symbol: character, size, font_name} = charkey;


        let mut canvas;
        let color = [1.0, 1.0, 1.0, 1.0];

        let pixel_size = size as f32 * 3.00;

        unsafe{
            //construct a char buffer
            let mut char_buffer;
            let cwidth;
            let cheight;
            let scale;
            {//NOTE
             //this accounts for about 10% of character rendering time.
             //If we want an easy speed up we can save the results to a global buffer  map
             // can only add to it when there is a new character being renedered
             // however if we build in release mode it doesn't really matter
                let mut x0 = 0i32;
                let mut x1 = 0i32;
                let mut y0 = 0i32;
                let mut y1 = 0i32;
                let mut ascent = 0;
                let mut descent = 0;

                let font;
                //TODO 
                //make more robust to non-exsistant font names 
                if font_name.is_none(){
                    font = fontlib["default"];
                } else{
                    font = fontlib[&font_name.unwrap()];
                }
                initfont(font);

                stbtt_GetFontVMetrics( &mut GLOBAL_FONTINFO as *mut stbtt_fontinfo,
                                      &mut ascent as *mut i32,
                                      &mut descent as *mut i32, null_mut());
                scale = stbtt_ScaleForPixelHeight(&GLOBAL_FONTINFO as *const stbtt_fontinfo, pixel_size as f32);
                let baseline = (ascent as f32 * scale ) as i32;

                cwidth = (scale * (ascent - descent) as f32 ) as usize + 4;
                cheight = (scale * (ascent - descent) as f32 ) as usize + 4;
                char_buffer = vec![0u8; cwidth * cheight];

                //render char to buffer
                stbtt_GetCodepointBitmapBoxSubpixel(&GLOBAL_FONTINFO as *const stbtt_fontinfo, character as u8, scale, scale, 0.0,0.0,
                                                    &mut x0 as *mut i32,
                                                    &mut y0 as *mut i32,
                                                    &mut x1 as *mut i32,
                                                    &mut y1 as *mut i32);
                //Sometimes x gets set to rediculous values
                stbtt_MakeCodepointBitmapSubpixel(  &GLOBAL_FONTINFO as *const stbtt_fontinfo,
                                                    &mut char_buffer[cwidth as usize * (baseline + y0) as usize + (5 + x0) as usize ] as *mut u8,
                                                     x1-x0+2, y1-y0, cwidth as i32, scale, scale,0.0, 0.0, character as i32);

                
                canvas = Bmp{w: cwidth as u32, h: cheight as u32, buffer: vec![0;4*cwidth*cheight]};
            }
            if character as u8 > 0x20{   //render char_buffer to main_buffer
                let x = 4;
                let y = 4;
                let buffer = canvas.buffer.as_mut_ptr() as *mut u32;
                let gwidth = canvas.w as usize;
                let gheight = canvas.h as usize;
                let offset = (x as usize + y as usize * gwidth) as usize;
                for i in 0..cheight{
                    for j in 0..cwidth{
                        if (j + i*gwidth + offset) > gwidth * gheight {continue;}

                        if j + x as usize  > gwidth {continue;}
                        if i + y as usize  > gheight {continue;}

                        let text_alpha = char_buffer[j + cwidth * (cheight - 1 - i)] as f32;
                        let a = color[3];
                        let r = (color[0] * text_alpha * a) as u32;
                        let g = (color[1] * text_alpha * a) as u32;
                        let b = (color[2] * text_alpha * a) as u32;

                        let dst_rgb = buffer.offset( (j + i*gwidth + offset) as isize);
                        //TODO
                        //We need to clap these values
                        let _r = (*(dst_rgb as *const u8).offset(0) as f32 * (255.0 - text_alpha * a )/255.0 ) as u32;
                        let _g = (*(dst_rgb as *const u8).offset(1) as f32 * (255.0 - text_alpha * a )/255.0 ) as u32;
                        let _b = (*(dst_rgb as *const u8).offset(2) as f32 * (255.0 - text_alpha * a )/255.0 ) as u32;
                        let _a = (*(dst_rgb as *const u8).offset(3) as f32 * (255.0 - text_alpha * a )/255.0 ) as u32;

                        *buffer.offset( (j + i*gwidth + offset) as isize) = 0x00000000 + ( (text_alpha as u32 + _a) << 24) + (b+_b << 16) + (g+_g << 8) + r+_r;
                    }
                }
            }
        }
        let sw = canvas.w as f32 / self.texture.width() as f32;
        let sh = canvas.h as f32 / self.texture.height() as f32; 
        if self.cursor[0] + sw > 1.0{
            self.cursor[0]  = -1.0;
            self.cursor[1] += self.current_maxdepth;
        }


        let mut renderer = Renderer{ display: display, target: &mut self.texture.as_surface(), indices, program};
        gl_drawbmp( &mut renderer, &canvas, self.cursor[0], self.cursor[1], sw, sh, None );

        self.map.insert( _charkey, [(self.cursor[0]+1.0)/2.0, (self.cursor[1]+1.0)/2.0, sw/2.0, sh/2.0] );

        self.cursor[0] += sw;
        if sh > self.current_maxdepth{ self.current_maxdepth = sh; }
        
    }
}

pub fn get_advance(character: char, size: f32)->f32{unsafe{
    if GLOBAL_FONTINFO.data == null_mut() {
        println!("Global font has not been set.");
        return -1.0;
    }
    let mut adv = 0;
    let scale = stbtt_ScaleForPixelHeight(&GLOBAL_FONTINFO as *const stbtt_fontinfo, size);
    stbtt_GetCodepointHMetrics(&GLOBAL_FONTINFO as *const stbtt_fontinfo, character as i32, &mut adv as *mut i32, null_mut());
    return adv as f32 * scale;
}}

fn draw_char<T: glium::Surface>( renderer: &mut Renderer<T>, fontlib: &FontLib, charmap: &mut CharMap, character: char, font: Option<String>, size: u32, x: f32, y: f32, color: [f32; 4])->f32{
     
    let mut _font;
    //TODO 
    //make more robust to non-exsistant font names 
    if font.is_none(){
        _font = fontlib["default"];
    } else{
        _font = fontlib[&font.clone().unwrap()];
    }
    initfont( _font );

    //TODO 
    //make more robust to non-exsistant character/size/font keys are generated upon request 
    let charkey = CharKey{ symbol: character, size: size, font_name: font };
    let charvalue;

    if charmap.map.contains_key(&charkey){
        charvalue = charmap.map[&charkey];
    } else {
        charmap.insert( renderer.display, renderer.program, renderer.indices, charkey.clone(), fontlib); 
        charvalue = charmap.map[&charkey];
    }

    gl_drawtexture(renderer, &charmap.texture, x, y,  2.0*charvalue[2]*charvalue[2]/charvalue[3], 2.0*charvalue[3], None, Some(charvalue), Some(color));

    let pixel_size = size as f32 * 3.00;
    //return get_advance(character, pixel_size) * 1.0 / charmap.texture.width() as f32;
    return get_advance(character, pixel_size) * 1.0 / renderer.target.get_dimensions().0 as f32;
}

fn draw_string<T: glium::Surface>( renderer: &mut Renderer<T>, fontlib: &FontLib, charmap: &mut CharMap, string: &str, font: Option<String>, size: u32, 
                                   x: f32, y: f32, color: [f32; 4], _max_length: Option<f32>)->f32{
    let mut delta_x = 0.0;
    let mut delta_y = 0.0;
    let max_length = _max_length.unwrap_or(std::f32::MAX);

    let const_delta_y = size as f32 * 3.00 * 0.99 / renderer.target.get_dimensions().1 as f32;

    for _char in string.chars(){
        delta_x += draw_char( renderer, fontlib, charmap, _char, font.clone(), size, x + delta_x, y - delta_y, color.clone()); 
        if delta_x > max_length{
            delta_x = 0.0;
            delta_y += const_delta_y; //size as f32 * 3.00 * 0.99 / renderer.target.get_dimensions().1 as f32;
        }
    }
    return delta_y + const_delta_y;
}













#[derive(PartialEq, Clone)]
pub enum ButtonStatus{
    Up,
    Down,
    Default
}
pub struct MouseInfo{
    pub x: i32,
    pub y: i32,

    pub lbutton: ButtonStatus,
    pub old_lbutton: ButtonStatus,

    pub rbutton: ButtonStatus,
    pub old_rbutton: ButtonStatus,

    pub wheel: isize,
    pub wheel_delta: i32,
}
impl MouseInfo{
    pub fn new()->MouseInfo{
        MouseInfo{
            x: 0,
            y: 0,
            lbutton: ButtonStatus::Default,
            old_lbutton: ButtonStatus::Default,
            rbutton: ButtonStatus::Default,
            old_rbutton: ButtonStatus::Default,
            wheel: 0,
            wheel_delta: 0,
        }
    }
}



struct Renderer<'a, T: glium::Surface>{
    display: &'a glium::Display,
    target:  &'a mut T,
    indices: &'a glium::index::NoIndices,
    program: &'a glium::Program,
}

#[derive(Copy, Clone)] 
struct Vertex {
    pub position: [f32; 2],
    pub tex_coords: [f32; 2],
}
implement_vertex!(Vertex, position, tex_coords);

fn generate_plane( x1: f32, x2: f32, y1: f32, y2: f32, display: &glium::Display)->glium::VertexBuffer<Vertex>{
    let vertex1 = Vertex{ position: [x1, y1], tex_coords: [0.0, 0.0]};
    let vertex2 = Vertex{ position: [x1, y2], tex_coords: [0.0, 1.0]};
    let vertex3 = Vertex{ position: [x2, y2], tex_coords: [1.0, 1.0]};
    let vertex4 = Vertex{ position: [x2, y1], tex_coords: [1.0, 0.0]};
    let vertex5 = Vertex{ position: [x2, y2], tex_coords: [1.0, 1.0]};
    let vertex6 = Vertex{ position: [x1, y1], tex_coords: [0.0, 0.0]};
    let shape = vec![vertex1, vertex2, vertex3, vertex4, vertex5, vertex6];

    return glium::VertexBuffer::new(display, &shape).expect("could not generate vertex buffer");
}

fn generate_plane_ex( pos_rect: [f32; 4], tex_rect: [f32; 4], display: &glium::Display)->glium::VertexBuffer<Vertex>{
    let pos_x1 = pos_rect[0];
    let pos_y1 = pos_rect[1];
    let pos_x2 = pos_rect[0] + pos_rect[2];
    let pos_y2 = tex_rect[1] + pos_rect[3];

    let tex_x1 = tex_rect[0];
    let tex_y1 = tex_rect[1];
    let tex_x2 = tex_rect[0] + tex_rect[2];
    let tex_y2 = tex_rect[1] + tex_rect[3];


    let vertex1 = Vertex{ position: [pos_x1, pos_y1], tex_coords: [tex_x1, tex_y1]};
    let vertex2 = Vertex{ position: [pos_x1, pos_y2], tex_coords: [tex_x1, tex_y2]};
    let vertex3 = Vertex{ position: [pos_x2, pos_y2], tex_coords: [tex_x2, tex_y2]};
    let vertex4 = Vertex{ position: [pos_x2, pos_y1], tex_coords: [tex_x2, tex_y1]};
    let vertex5 = Vertex{ position: [pos_x2, pos_y2], tex_coords: [tex_x2, tex_y2]};
    let vertex6 = Vertex{ position: [pos_x1, pos_y1], tex_coords: [tex_x1, tex_y1]};
    let shape = vec![vertex1, vertex2, vertex3, vertex4, vertex5, vertex6];

    return glium::VertexBuffer::new(display, &shape).expect("could not generate vertex buffer");
}


fn main(){
    use std::env;
    let args: Vec<String> = env::args().collect();
    println!("{:?}", args);

    let dll_source_path;
    if args.len() < 2{
        //TODO make this colored text
        println!("Wrong number of parameters. Plz provide source file path");
        println!("how to use; ./NAME_OF_THIS_PROGRAM   source.rs  source.os(optional)");
        
        dll_source_path = "examples/test.rs";
        //return;
    } else {
        dll_source_path = &args[1];
    }


    if std::path::Path::new(dll_source_path).exists() == false{
        //TODO
        //check to make sure we are checking for a file not a directory.
        println!("Path '{}' does not exist.", dll_source_path);
        return;
    }


    let mut dll_path = "".to_string();

    if args.len() != 3{
      
        if cfg!(target_os = "windows"){
            let mut _dll_path = std::path::PathBuf::from(dll_source_path);
            _dll_path.set_extension("dll");
            dll_path = _dll_path.to_str().expect("Could not convert dll path to string!!!").to_string();
        }
        else if cfg!(target_os = "linux"){

            let _split_path : Vec<&str> = dll_source_path.split("/").collect();
            for (i, it) in _split_path.iter().enumerate(){

                if i < _split_path.len() -1 {
                    dll_path = dll_path + *it + "/";
                } else {
                    let l = it.len();
                    dll_path = dll_path + "lib" + &it[..l-3] + ".so";
                }
            }

        } else{
            panic!("Operating system not supported!!");
        }
    } else { //NOTE: If dll path is given
        dll_path += &args[2];
    }

    let copy_dll_path;
    if cfg!(target_os = "windows") {
        copy_dll_path = "temp.dll".to_string();
    } else{
        copy_dll_path = "temp.so".to_string();
    }

    let current_path = std::env::current_dir().unwrap();
    let mut str_current_path = current_path.to_str().unwrap().to_string();
    str_current_path += "/";
    str_current_path += &copy_dll_path;
    std::fs::copy(&dll_path, &copy_dll_path).expect("Could not copy dll");
    let mut app = open_lib(&str_current_path, dynamic_lib_loading::RTLD_LAZY).expect("Library could not be found.");
    //let mut app = open_lib(&copy_dll_path, dynamic_lib_loading::RTLD_LAZY).expect("Library could not be found.");








    //Window and event loop initialization

    let mut event_loop = glutin::EventsLoop::new();
    let monitor = event_loop.get_primary_monitor();
    let glutin::dpi::PhysicalSize{width: monitor_width, height: monitor_height} = monitor.get_dimensions();

    let _window = glutin::WindowBuilder::new()
                    .with_dimensions(glutin::dpi::LogicalSize{width:monitor_width / 2.0, height:monitor_height - 70.0}) //TODO Not robust and 
                    .with_title("tg_notebook");                                                                         //should be different per os and user config

    let mut windowinfo = WindowInfo{focused: true, width: monitor_width / 2.0,
                                    height: monitor_height - 70.0 };

    let context = glutin::ContextBuilder::new();
    let display = glium::Display::new(_window, context, &event_loop).unwrap();


    //NOTE use this to change the cursor we will need when we move and resize
    // https://docs.rs/glutin/0.21.0/glutin/struct.Window.html#method.set_cursor
    display.gl_window().window().set_position(glutin::dpi::LogicalPosition{x:monitor_width/2.0, y:0.0}); 




    //Initializing instruction buffer and getting initial functions from dll
    let mut instructionbuffer = InstructionBuffer::new();
    get_function_from_source( dll_source_path, &app, &mut instructionbuffer, &display );





    //Font initialization
    let mut fontlib = FontLib::new();
    unsafe{
        fontlib.insert("default".to_string(), DEFAULT_FONT_BUFFER);
        initfont( fontlib["default"] ); 
    }
    let mut charmap = CharMap::new(&display);






 

    //NOTE
    //original source code stolen from
    //https://github.com/glium/glium/blob/master/examples/tutorial-10.rs
    let vertex_shader_src = r#"
        #version 140
        
        in vec2 position;
        in vec2 tex_coords;
        out vec2 v_tex_coords;

        uniform mat4 perspective;
        uniform mat4 matrix;
       
        void main(){
            v_tex_coords = tex_coords;
            gl_Position = perspective * matrix * vec4(position, 0.0, 1.0);
        } 
    "#;

    let fragment_shader_src = r#"
        #version 140
    
        in vec2 v_tex_coords;
        out vec4 color;
      
        uniform sampler2D tex;
        void main(){
            color = texture(tex, v_tex_coords);
        }
    "#;

    set_windoinfo(windowinfo);
    let mut mouseinfo = MouseInfo::new();



    let program = glium::Program::from_source(&display, vertex_shader_src, fragment_shader_src, None).expect("Could not compile shaders");
    let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);


    //Rasterizing most used characters
    for it in "abcdefghijklminopqrstuvwxyz+=?/0123456789".chars(){
        charmap.insert(&display, &program, &indices, CharKey{symbol: it, size: 22, font_name: None}, &fontlib);
        charmap.insert(&display, &program, &indices, CharKey{symbol: it.to_ascii_uppercase(), size: 22, font_name: None}, &fontlib);
    }

    for it in "abcdefghijklminopqrstuvwxyz+=?/0123456789".chars(){
        charmap.insert(&display, &program, &indices, CharKey{symbol: it, size: 16, font_name: None}, &fontlib);
        charmap.insert(&display, &program, &indices, CharKey{symbol: it.to_ascii_uppercase(), size: 16, font_name: None}, &fontlib);
    }


    let mut globalstorage = GlobalStorage::new();
    let mut interactive_info :InteractiveInfo= Default::default(); //TODO


    let mut exit = false;
    //CLEANUP remove me 

    let mut last_modified = std::fs::metadata(&dll_path).unwrap().modified().unwrap();
    let mut last_dll_size = std::fs::metadata(&dll_path).unwrap().len();
    let mut debug_infostate = InfoState::new();
    debug_infostate.most_recent_load_stamp = format!("{:?}", std::time::Instant::now());

    'gameloop: loop{


        if let Ok((Ok(modified), new_dll_size_bytes)) = std::fs::metadata(&dll_path).map(|m| (m.modified(), m.len())) 
        {
            if modified > last_modified && last_dll_size == new_dll_size_bytes 
            { 
                //NOTE(Question)
                //If we close lib do we retain previously loaded function pointers?

                debug_infostate.most_recent_load_stamp = format!("{:?}", std::time::Instant::now());
                println!("{}", "Updating dll!!!");
                close_lib(&app);
                drop(app);

                //////////////////////
                //TODO this is stupid 
                let mut i_loop = 0;
                loop {
                    match std::fs::copy(&dll_path, &copy_dll_path){
                        Ok(_) =>{ break; },
                        Err(code) =>{  
                            i_loop += 1;
                            if false {
                                panic!("dll could not be copied failed with non-supported error {}", code);  
                            }
                        }
                    }
                }
                println!("How many loops where needed until I could copy the file? {}", i_loop);
                //the above is stupid
                //////////////////////

                app = open_lib(&str_current_path, dynamic_lib_loading::RTLD_LAZY).expect("Library could not be found.");
                //app = open_lib(&copy_dll_path, dynamic_lib_loading::RTLD_LAZY).expect("Library could not be found.");
                get_function_from_source( dll_source_path, &app, &mut instructionbuffer, &display);
                last_modified = modified;

            }
            if modified > last_modified && new_dll_size_bytes > 500_000{ last_dll_size = new_dll_size_bytes; }
        } 





        mouseinfo.wheel_delta = 0;
        event_loop.poll_events(|event| {
            match event {
                glutin::Event::DeviceEvent{ event: devent, ..} => { 
                    match devent { 
                        glutin::DeviceEvent::Key(kb) =>{ 
                            //NOTE 
                            //scancode 1 is the code for Esc
                            if kb.scancode == 1 && windowinfo.focused {exit = true;};
                        },
                        _=> () 
                    }
                },
                glutin::Event::WindowEvent { event: wevent, .. } => match wevent {
                    glutin::WindowEvent::CloseRequested => exit = true,
                    glutin::WindowEvent::Focused(_bool) => windowinfo.focused = _bool,
                    glutin::WindowEvent::Resized(pos) => {  windowinfo.width = pos.width; windowinfo.height = pos.height; },

                    glutin::WindowEvent::CursorMoved{position, ..} => {
                        let _pos : (i32, i32) = position.into();
                        mouseinfo.x = _pos.0;
                        mouseinfo.y = windowinfo.height as i32 - _pos.1;
                    },
                    glutin::WindowEvent::MouseInput{button, state, ..} => {
                        match button{
                            glutin::MouseButton::Left=>{
                                if state == glutin::ElementState::Pressed{
                                    mouseinfo.lbutton = ButtonStatus::Down;
                                } else{
                                    mouseinfo.lbutton = ButtonStatus::Up;
                                }
                            },
                            glutin::MouseButton::Right=>{
                                if state == glutin::ElementState::Pressed{
                                    mouseinfo.rbutton = ButtonStatus::Down;
                                } else{
                                    mouseinfo.rbutton = ButtonStatus::Up;
                                }
                            },
                            _=>{/*TODO*/}
                        }
                    },
                    glutin::WindowEvent::MouseWheel{delta, ..} => {
                        match delta{
                            glutin::MouseScrollDelta::LineDelta(_, y) => {
                                mouseinfo.wheel += y as isize;
                                mouseinfo.wheel_delta = y as _;
                            },
                            glutin::MouseScrollDelta::PixelDelta(lgpos) => {
                                let _pos : (i32, i32) = lgpos.into();
                                mouseinfo.wheel += _pos.1 as isize;
                                mouseinfo.wheel_delta = _pos.1 as i32;
                            }
                        }
                    },
                    glutin::WindowEvent::ReceivedCharacter(c) => {
                        interactive_info.text_key_pressed = c; //TODO there is a difference between received character and key pressed
                    },
                    _=>{}
                },
                _=>{}
            }
        });


        set_windoinfo(windowinfo);


        logic_panel(&mut debug_infostate, &mut instructionbuffer, &mouseinfo);

        
        for i in 0..instructionbuffer.ids.len(){//Run new dll functions
            if instructionbuffer.initialized[i] == false || instructionbuffer.interactive[i]{ 
                let op_error = instructionbuffer.fns[i]( &mut instructionbuffer.render_instructions[i], 
                                                         &mut globalstorage, 
                                                         &mut instructionbuffer.localstorage[i], 
                                                         & interactive_info); //TODO handle errors
                instructionbuffer.initialized[i] = true;
                instructionbuffer.interactive[i] = instructionbuffer.localstorage[i].interactive;
                match op_error {
                    Ok(_) => {},
                    Err(e) => {
                        debug_infostate.errors.push(e.clone());
                        instructionbuffer.errors[i] = e; 
                    }
                }
            }
        }


        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 0.0, 1.0);
        {

            let mut renderer = Renderer{  display: &display, 
                                      target:  &mut target, 
                                      indices: &indices,
                                      program: &program,
                                   }; 

           render_panel( &mut instructionbuffer, &mut renderer, &mouseinfo, &fontlib, &mut charmap);
           render_ui( &mut debug_infostate, &instructionbuffer, &mut renderer, &mouseinfo, &fontlib, &mut charmap);
        }

        target.finish().unwrap();
        if exit == true{break 'gameloop} 
    }


}





fn logic_panel( state: &mut InfoState, instructionbuffer: &mut InstructionBuffer, mouseinfo: &MouseInfo,){
    for i in 0..instructionbuffer.len(){
       if instructionbuffer.pos_rect[i][0] == 0.0 && 
          instructionbuffer.pos_rect[i][1] == 0.0 && 
          instructionbuffer.pos_rect[i][2] == 0.0 && 
          instructionbuffer.pos_rect[i][3] == 0.0 {
        
              continue;
        }
        //Bring sub frame into focus
        if mouseinfo.lbutton == ButtonStatus::Down && 
           in_rect( mouseinfo.x as i32, mouseinfo.y as i32,  
           convert_screen_coor_to_pixel_corr(instructionbuffer.pos_rect[i]) ) {
            
            instructionbuffer.infocus[i] = true;
            state.activated_function = instructionbuffer.ids[i].clone();
            for j in 0..instructionbuffer.len() {
                if i !=j { instructionbuffer.infocus[j] = false; }
            }
        }
        if instructionbuffer.infocus[i] {

            /////////////////////////
            // TODO not good
            if instructionbuffer.max_println_y[i] > 1.0{
                instructionbuffer.println_y[i] += (mouseinfo.wheel_delta as f32) * 1.0/10.0;
                if instructionbuffer.println_y[i] < -0.11 {
                    instructionbuffer.println_y[i] = 0.0;
                }
                else if 1.11 - instructionbuffer.println_y[i] < 0.0  {
                    instructionbuffer.println_y[i] = 1.0;
                }
            }
            /////////////////////////

        }
    }
}

fn render_panel<T: glium::Surface>( instructionbuffer: &mut InstructionBuffer, renderer: &mut Renderer<T>, mouseinfo: &MouseInfo, fontlib: &FontLib, charmap: &mut CharMap ){
    //TODO
    //clean up these hard coded values
    let height = DEFAULT_TEXTURE_HEIGHT_FRAC;
    let header_height = 0.05;
    let top = 0.945;



    for (i, it) in instructionbuffer.textures.iter_mut().enumerate(){
        let _i = (i + 1) as f32;
        let mut max_println_y;


        gl_drawrect(renderer, -0.7, top - _i * header_height - (_i - 1.0) * height, DEFAULT_TEXTURE_WIDTH_FRAC, header_height, [0.07, 0.07, 0.07, 1.0], None);
        draw_string(renderer, fontlib, charmap, &instructionbuffer.ids[i], None, 16, -0.68, top - _i * header_height - (_i - 1.0) * height - 0.01, [1.0; 4], None);
        {
            let mut surface = it.as_surface();
            surface.clear_color(C_LIGHTBLACK[0], C_LIGHTBLACK[1], C_LIGHTBLACK[2], C_LIGHTBLACK[3]);
            let mut _renderer = Renderer{ display: renderer.display, target: &mut surface, indices: renderer.indices, program: renderer.program};


            for renderable in instructionbuffer.render_instructions[i].buffer.iter(){
                let rendertype = renderable.rendertype.clone();
                
                match rendertype {
                    RenderType::Image => {
                        //TODO
                    },
                    RenderType::Rectangle => {//TODO outline vs filled
                        let color = [renderable.color[0], renderable.color[1],renderable.color[2], renderable.alpha*0.8];
                        gl_drawrect( &mut _renderer, renderable.x, renderable.y, renderable.width * DEFAULT_TEXTURE_HEIGHT_FRAC/DEFAULT_TEXTURE_WIDTH_FRAC , renderable.height, color, None);
                    },
                    RenderType::String => { 
                        //TODO
                        //Debug
                        let color = [renderable.color[0], renderable.color[1],renderable.color[2], renderable.alpha*0.8];
                        draw_string( &mut _renderer, fontlib, charmap, &renderable.char_buffer,
                                     None, renderable.font_size, renderable.x, renderable.y , color, None);
                        
                    },
                    RenderType::PrintString => {},
                    RenderType::Empty => {},
                }
            }
            //Draw print statements to texture
            let x = -0.99;
            let mut y = 0.85; 
            for renderable in instructionbuffer.render_instructions[i].buffer.iter(){

                let mut delta_println_y = 0.0;//TODO this is bad remove me
                if instructionbuffer.max_println_y[i] > 2.0 {
                    delta_println_y = 1.0 + instructionbuffer.println_y[i] - instructionbuffer.max_println_y[i];
                }
                if renderable.rendertype == RenderType::PrintString{
                    let color = [renderable.color[0], renderable.color[1], renderable.color[2], renderable.alpha];
                    let delta = draw_string( &mut _renderer, fontlib, charmap, &renderable.char_buffer,
                                             None, renderable.font_size, x, y - delta_println_y , color, Some(1.85));
                    y -= delta;
                }

            }
            {//TODO
             // render errors
             //   draw_string( &mut _renderer, fontlib, charmap, &instructionbuffer.errors[i],
             //               None, renderable.font_size, x, y - delta_println_y , C_RED, Some(1.85));
            }
            max_println_y = y-0.15;

            instructionbuffer.max_println_y[i] = max_println_y.abs();
            if max_println_y.abs() > 1.0{//draw scroll bar

                gl_drawrect(&mut _renderer, 0.97, -1.0, 0.1, 2.0, C_GREY, None);
                gl_drawrect(&mut _renderer, 0.97, -1.0+instructionbuffer.println_y[i], 0.1, 2.0/(max_println_y.abs()), C_GREEN, None);
            }

        }

        let pos_y = top - _i * header_height - _i * height;

        gl_drawtexture(renderer, it, -0.7, pos_y, 1.0, 1.0, None, None, None);
        instructionbuffer.pos_rect[i][0] = -0.7;
        instructionbuffer.pos_rect[i][1] = pos_y;
        instructionbuffer.pos_rect[i][2] = DEFAULT_TEXTURE_WIDTH_FRAC;
        instructionbuffer.pos_rect[i][3] = DEFAULT_TEXTURE_HEIGHT_FRAC;
    }



}

struct InfoState{
    activated_function: String,
    errors: Vec<String>,
    scrollthumb_rect: [f32; 4],
    most_recent_load_stamp: String,
}

impl InfoState{
    pub fn new()->InfoState{
        InfoState{
            activated_function: String::with_capacity(250),
            errors: vec![],
            scrollthumb_rect : [0.98, -0.92, 0.15, 1.82],
            most_recent_load_stamp: String::new(),
        }
    }
}

fn render_ui<T: glium::Surface>( state: &mut InfoState, instructionbuffer: &InstructionBuffer, renderer: &mut Renderer<T>, mouseinfo: &MouseInfo, fontlib: &FontLib, charmap: &mut CharMap ){

    gl_drawrect(renderer, -0.7, -1.0, 0.01, 2.0, C_GREY, None); // Line that divides the list of functions and the function outputs
    for (i, id) in instructionbuffer.ids.iter().enumerate(){
        draw_string(renderer, fontlib, charmap, id , None, 16, -0.98, 0.88-i as f32*0.05, C_WHITE, None);
    }
    

    let full_length = instructionbuffer.len() as f32 * DEFAULT_TEXTURE_HEIGHT_FRAC;
    state.scrollthumb_rect[3] = 1.92/full_length;
    gl_drawrect(renderer, 0.98, -0.94, 0.1, 1.92, C_GREY, None);     //scroll bar
    {
        let [x, y, w, h] = state.scrollthumb_rect;
        gl_drawrect(renderer, x, y, w, h, C_PURPLE, None);  //scroll thumb 
    }

    gl_drawrect(renderer, -1.0, 0.94, 2.0, 0.06, C_GREY, None); // header rect
    if state.activated_function.len() == 0 { //Denote activated function
        draw_string(renderer, fontlib, charmap, "----- ", None, 16, -0.01, 0.94, C_WHITE, None);
    } else {
        let string = &state.activated_function;
        draw_string(renderer, fontlib, charmap, string, None, 16, -0.01, 0.94, C_WHITE, None);
    }

    { //Errors
    
        //TODO 
        // add interactions. User should be able to open up this to a panel to see all errors from the last compile and errors from previous  compiles if they want.
        gl_drawrect(renderer, -1.0, -1.0, 2.0, 0.06, C_GREY, None); // footer
        if state.errors.len() != 0 {
            let string = format!("{}", state.errors[state.errors.len() - 1]);
            draw_string(renderer, fontlib, charmap, &string, None, 16, -0.01, -1.005, C_RED, None);
        } else {
            draw_string(renderer, fontlib, charmap, &state.most_recent_load_stamp , None, 16, -0.01, -1.005, C_BLACK, None);
        } 
    }
}




pub fn convert_screen_coor_to_pixel_corr(rect: [f32;4])-> [i32; 4]{
    let WindowInfo{focused, width, height} = clone_windowinfo();
    
    let mut rt = [0i32; 4];
    rt[0] = ( ( 1.0 + rect[0] as f64 ) * width / 2.0) as i32;
    rt[1] = ( ( 1.0 + rect[1] as f64 ) * height / 2.0) as i32;
    rt[2] = ( rect[2] as f64 * width / 2.0) as i32;
    rt[3] = ( rect[3] as f64 * height / 2.0) as i32;
   
    rt
}


//TODO TEMP
struct Bmp{
    pub w: u32,
    pub h: u32,
    pub buffer: Vec<u8>,
}

//TODO
//Think about this
fn gl_drawbmp<T: glium::Surface>( renderer: &mut Renderer<T>, bmp: &Bmp, x: f32, y: f32, sw: f32, sh: f32, perspective: Option<[[f32;4];4]> )->glium::texture::Texture2d{
    let Renderer{display, target, indices, program} = renderer;

    let w = bmp.w;
    let h = bmp.h;

    let ratio = w as f32 / h as f32;

    let image = glium::texture::RawImage2d::from_raw_rgba(bmp.buffer.clone(), (w, h));//I would rather not clone but what can you do.
    let texture = glium::texture::Texture2d::new(*display, image).expect("we could not make the texture");

    let mut _perspective = perspective.unwrap_or(
                          [  [1.0, 0.0, 0.0, 0.0,],
                             [0.0, 1.0, 0.0, 0.0,],
                             [0.0, 0.0, 1.0, 0.0,],
                             [0.0, 0.0, 0.0, 1.0f32]]
                          );
    //TODO
    //need an x-axis and y-axis scaling term
    let     transform = Matrix4::new(sw, 0.0, 0.0, 0.0,
                                     0.0, sh, 0.0, 0.0,
                                     0.0, 0.0, 1.0, 0.0,
                                       x,   y, 1.0, 1.0);
     

    fn populate_array_using_matrix(in_matrix: &Matrix4<f32>, out_array: &mut [[f32;4];4]){
    //TODO
    //move out at some point
        for (i, it) in in_matrix.iter().enumerate(){
            let _i = i / 4;
            let _j = i % 4;
            out_array[_j][_i] = *it;
        }
    }

    let mut _matrix = [[0.0f32; 4]; 4];
    populate_array_using_matrix( &transform, &mut _matrix);
    {
        let uniforms = uniform!{ 
            perspective: _perspective,	
            matrix: _matrix,
            tex: &texture,
        };
        
        //TODO Is this ok? it's prob slow right?
        let vertexbuffer = generate_plane(0.0, ratio, 0.0, 1.0, display);

        target.draw(&vertexbuffer, *indices, program, &uniforms, &Default::default()).unwrap();
    }

    return texture;
}

fn gl_drawrect<T: glium::Surface>(renderer: &mut Renderer<T>, x: f32, y: f32, sw: f32, sh: f32, color: [f32; 4], perspective: Option<[[f32;4];4]>){

    let texture = glium::texture::Texture2d::empty(renderer.display, 100, 100).expect("rect texture was not able to be generated");
    {
        let [r, g, b, a] = color;
        let mut surface = texture.as_surface();
        surface.clear_color( r, g, b, a);
    }
    let (_w, _h) = renderer.target.get_dimensions();
    gl_drawtexture(renderer, &texture, x, y, sw * (_w as f32)/100.0, sh * (_h as f32) / 100.0, None, None, None);

}



fn gl_drawtexture<T: glium::Surface>(renderer: &mut Renderer<T>, texture: &glium::texture::Texture2d, x: f32, y: f32, sw: f32, sh: f32, perspective: Option<[[f32;4];4]>, subrect: Option<[f32;4]>, color: Option<[f32; 4]>){
    let Renderer{display, target, indices, program} = renderer;

    let w = texture.width();
    let h = texture.height();

    let (display_w, display_h) = target.get_dimensions();
    let ratio_w = w as f32 / display_w as f32;
    let ratio_h = h as f32 / display_h as f32;

    let mut _perspective = perspective.unwrap_or(
                          [  [1.0, 0.0, 0.0, 0.0,],
                             [0.0, 1.0, 0.0, 0.0,],
                             [0.0, 0.0, 1.0, 0.0,],
                             [0.0, 0.0, 0.0, 1.0f32]]
                          );
    //TODO
    //need an x-axis and y-axis scaling term
    let     transform = Matrix4::new( sw, 0.0, 0.0, 0.0,
                                     0.0,  sh, 0.0, 0.0,
                                     0.0, 0.0, 1.0, 0.0,
                                       x,   y, 1.0, 1.0);
     

    fn populate_array_using_matrix(in_matrix: &Matrix4<f32>, out_array: &mut [[f32;4];4]){
    //TODO
    //move out at some point
        for (i, it) in in_matrix.iter().enumerate(){
            let _i = i / 4;
            let _j = i % 4;
            out_array[_j][_i] = *it;
        }
    }

    let mut _matrix = [[0.0f32; 4]; 4];
    populate_array_using_matrix( &transform, &mut _matrix);
    {
        let uniforms = uniform!{ 
            perspective: _perspective,	
            matrix: _matrix,
            tex: texture,
        };
        
        //TODO This is temp
        let vertexbuffer = generate_plane_ex([0.0, 0.0, ratio_w, ratio_h], subrect.unwrap_or([0.0, 0.0, 1.0, 1.0]), display);

        let mut draw_params : glium::draw_parameters::DrawParameters =  Default::default();
        draw_params.blend = glium::Blend::alpha_blending();

        if color.is_some(){
            let [r, g, b, a] = color.unwrap();
            draw_params.blend.color = glium::BlendingFunction::Addition{ source: glium::LinearBlendingFactor::ConstantColor, destination: glium::LinearBlendingFactor::OneMinusSourceAlpha };
            draw_params.blend.constant_value = (r, g, b, a);
        }

        target.draw(&vertexbuffer, *indices, program, &uniforms, &draw_params).unwrap();
    }
}

pub fn in_rect(x: i32, y: i32, rect: [i32;4])->bool{
    let mut rt = true;
    if x < rect[0]{
        rt = false;
    }
    if y < rect[1]{
        rt = false;
    }
    if x > rect[0] + rect[2]{
        rt = false;
    }
    if y > rect[1] + rect[3]{
        rt = false;
    }
    return rt;
}
pub fn in_rectf32(x: f32, y: f32, rect: [f32;4])->bool{
    let mut rt = true;
    if x < rect[0]{
        rt = false;
    }
    if y < rect[1]{
        rt = false;
    }
    if x > rect[0] + rect[2]{
        rt = false;
    }
    if y > rect[1] + rect[3]{
        rt = false;
    }
    return rt;
}





fn get_function_from_source( dll_source_path: &str, app: &DyLib, instructionbuffer: &mut InstructionBuffer, display: &glium::Display ){

    use std::io::prelude::*;
    let mut f = std::fs::File::open(dll_source_path).expect("Source file does not exist.");
    let mut str_file = String::new();
    f.read_to_string(&mut str_file).expect("could not read file to string.");
    let mut primed_ud_fn_name = false;
    let mut eat_source = false;
    let mut function_names = vec![];
    let mut function_scrs = vec![];


    //TODO
    //we need to check for changes in the source
    for it in str_file.lines(){
        if primed_ud_fn_name {

            //NOTE
            //primed meaning that the previous line contains '#[no_mangle]' this is one of two keys
            //we use to determine if a we are to try and load a function.  '#[no_mangle]' is needed
            //so that we can find the function by name within the dll/so file.
            primed_ud_fn_name = false;
            let mut function_name_buffer = vec!['U', 'D', '_']; //TODO Current prefix for function of interest.
            let prefix = "fn UD_";


            if it.contains(prefix){//NOTE Should maybe be begins instead of constains....

                let seq_must_meet :Vec<char>= prefix.chars().collect();
                let mut seq_cursor = 0;
                let mut seq_good = false;


                for jt in it.chars(){
                    if seq_good == false { 
                        if seq_must_meet[seq_cursor] == jt { 
                            seq_cursor += 1;
                            if seq_cursor == seq_must_meet.len() { seq_good = true;}
                        }
                    } else {
                        if jt != '(' { function_name_buffer.push(jt); }
                        else { break; }
                    }
                }


            } 
            function_names.push( function_name_buffer.iter().collect::<String>());
            function_scrs.push( String::new());
        }
        if eat_source {
            //TODO
            //The following is not robust should count curly braces or something and make sure things are good.
            let cursor = function_scrs.len()-1;
            function_scrs[cursor] += it;
            function_scrs[cursor] += "\n";
            if "}" == it || "} " == it {
               eat_source = false; 
            }
        }
        if it == "#[no_mangle]" { 
            primed_ud_fn_name = true;
            eat_source = true;
        }
    } 

    for (i, it) in function_names.iter().enumerate(){
        match instructionbuffer.get_index(it){ //TODO I need to get the index of the function from the instructionbuffer
            Ok(index)=>{
                drop(instructionbuffer.fns[index]);
              

                let mut game_logic : UserFn = unsafe{ std::mem::transmute(get_fn(app, it).unwrap().as_mut()) };
                instructionbuffer.fns[index] = game_logic;

                if instructionbuffer.fns_source[index] != function_scrs[i]{//check if source has changed
                    instructionbuffer.fns_source[index]= function_scrs[i].clone();
                    instructionbuffer.initialized[index] = false;
                    instructionbuffer.render_instructions[index].clear();
                } else {
                    instructionbuffer.initialized[index] = true;//TODO check this
                    instructionbuffer.interactive[i] == instructionbuffer.localstorage[i].interactive;
                }
                continue; 
            },
            _=>{
                let (target_w, target_h) = display.get_framebuffer_dimensions();
                let texture = glium::texture::Texture2d::empty(display, (target_w as f32 * DEFAULT_TEXTURE_WIDTH_FRAC) as _,
                                                                        (target_h as f32 * DEFAULT_TEXTURE_HEIGHT_FRAC) as _ ).expect("texture could not be created.");

                let mut game_logic : UserFn = unsafe{ std::mem::transmute(get_fn(app, it).unwrap().as_mut()) };

                instructionbuffer.push(texture,
                                      it.to_string(),
                                      function_scrs[i].to_string(),
                                      game_logic); //TODO  we need provious loads source info so we can
                                      //diff to determine what's changed.
            }
        }
    }

}



