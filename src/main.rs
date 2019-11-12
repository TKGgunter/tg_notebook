#[macro_use]
extern crate glium;
extern crate nalgebra;
extern crate stb_tt_sys;

use nalgebra::core::Matrix4;
use glium::{glutin, Surface};

use stb_tt_sys::*;
use std::ptr::{null_mut, null};

//NOTE
//what is the time cost of loading various resolutions of textures all the time : it takes about 2 millis to generate a high res bmp
//To generate a texture and push that texture to the gpu it takes another 2-3 millisecs

//TODO
// + render text



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
            current_maxdepth: -1.0,
            
        }
    }
    pub fn insert(&mut self, display: &glium::Display, program: &glium::Program, indices: &glium::index::NoIndices, charkey: CharKey, fontlib: &FontLib)->Option<()>{
    //TODO
    //might need to change how we render to texture

        let _charkey = charkey.clone();
        let CharKey{symbol: character, size, font_name} = charkey;
        let mut canvas;
        let color = [1.0, 1.0, 1.0, 1.0];

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

                let mut font;
                if font_name.is_none(){
                    font = fontlib["default"];
                } else{
                    font = fontlib[&font_name.unwrap()];
                }
                initfont(font);

                stbtt_GetFontVMetrics( &mut GLOBAL_FONTINFO as *mut stbtt_fontinfo,
                                      &mut ascent as *mut i32,
                                      &mut descent as *mut i32, null_mut());
                scale = stbtt_ScaleForPixelHeight(&GLOBAL_FONTINFO as *const stbtt_fontinfo, size as f32);
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
        println!("{} {}", sw, sh);
        if self.cursor[0] + sw > 1.0{
            self.cursor[0]  = -1.0;
            self.cursor[1] += self.current_maxdepth;
        }


        let mut renderer = Renderer{ display: display, target: &mut self.texture.as_surface(), indices, program};
        gl_drawbmp( &mut renderer, &canvas, self.cursor[0], self.cursor[1], sw, sh, None);

        self.map.insert( _charkey, [self.cursor[0], self.cursor[1], sw, sh] );

        self.cursor[0] += sw;
        if self.cursor[1] + sh > self.current_maxdepth{ self.current_maxdepth = self.cursor[1] + sh; }
        
        return None;
    }
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
    target:  &'a mut T, //glium::Frame,
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


fn main(){
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
    let mut temp_state = TEMP_STATE{  init: false,
                                      box_in_focus: false, 
                                      x: 0.0, 
                                      y: 0.0,
                                      w: 0.0, 
                                      h: 0.0,

                                      prev_mouse: [0, 0],
                                      .. Default::default()
                                      };



    let program = glium::Program::from_source(&display, vertex_shader_src, fragment_shader_src, None).expect("Could not compile shaders");
    let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);

    charmap.insert(&display, &program, &indices, CharKey{symbol: 'Y', size: 64, font_name: None}, &fontlib);
    charmap.insert(&display, &program, &indices, CharKey{symbol: 'B', size: 64, font_name: None}, &fontlib);
    charmap.insert(&display, &program, &indices, CharKey{symbol: 'C', size: 64, font_name: None}, &fontlib);

    let mut exit = false;
    //CLEANUP remove me 
    let mut temp_texture : Option<glium::texture::Texture2d> = None;
    'gameloop: loop{
        let mut window_info = clone_windowinfo();

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
                    _=>{}
                },
                _=>{}
            }
        });


        set_windoinfo(windowinfo);


        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 0.0, 1.0);
        {

            let mut renderer = Renderer{  display: &display, 
                                      target:  &mut target, 
                                      indices: &indices,
                                      program: &program,
                                   }; 



            if temp_texture.is_none() {
                temp_texture = TEMP_FN(&mut temp_state, renderer, &mouseinfo, None);
            } else {
                TEMP_FN(&mut temp_state, renderer, &mouseinfo, temp_texture.as_mut());
            }


        }
        {

            let mut renderer = Renderer{  display: &display, 
                                      target:  &mut target, 
                                      indices: &indices,
                                      program: &program,
                                   }; 

            gl_drawtexture(&mut renderer, &charmap.texture, -0.95, -0.95, None);
        }

        target.finish().unwrap();
        if exit == true{break 'gameloop} 
    }


}

#[derive(Default)]
struct TEMP_STATE{
    init: bool,
    box_in_focus: bool,
    x: f32,
    y: f32,
    w: f32,
    h: f32, 

    prev_mouse: [i32; 2],
    rect1: [f32; 4],
}


fn TEMP_FN<T: glium::Surface>( state: &mut TEMP_STATE, mut renderer: Renderer<T>, mouseinfo: &MouseInfo, texture: Option<&mut glium::texture::Texture2d> )->Option<glium::texture::Texture2d>{
    let mut texture = glium::texture::Texture2d::empty(renderer.display, 100, 100).expect("we could not make the texture");
    {

        let mut surface = texture.as_surface();
        surface.clear_color( 1.0, 0.0, 0.0, 1.0);
    }
    gl_drawtexture(&mut renderer, &texture, -0.2, -0.2, None);
    
    return None;    
}

fn _2TEMP_FN<T: glium::Surface>( state: &mut TEMP_STATE, mut renderer: Renderer<T>, mouseinfo: &MouseInfo, texture: Option<&mut glium::texture::Texture2d> )->Option<glium::texture::Texture2d>{
    //Info
    //Scrolling example.

    if state.init == false{
        state.init = true;
        state.box_in_focus = false;
        state.w    = 1000.0;
        state.h    = 1000.0;


        state.rect1 = [0.0, 0.0, 0.4, 0.2];
    }

    let mut rect1 = Bmp{ w: 30,
                         h: 30,
                         buffer: vec![0x11; 4*30*30],
                       };

    state.rect1[1] += (mouseinfo.wheel_delta as f32) * 1.0/10.0;
    
    let [x, y, w, h] = state.rect1;
    gl_drawbmp( &mut renderer, &rect1, x, y, w, h, None );

    return None;
}


fn _TEMP_FN<T: glium::Surface>( state: &mut TEMP_STATE, mut renderer: Renderer<T>, mouseinfo: &MouseInfo, texture: Option<&mut glium::texture::Texture2d> )->Option<glium::texture::Texture2d>{
    //Info
    //Test function used to move a rectangle around the screen after a click


    if state.init == false{
        state.init = true;
        state.box_in_focus = false;
        state.w    = 1000.0;
        state.h    = 1000.0;
    }


    if texture.is_none(){
        let w = state.w as u32;
        let h = state.h as u32;

        let bmp = Bmp{ w: w, h: h, buffer: vec![150u8; (4*w*h) as usize]};
        return Some(gl_drawbmp(&mut renderer, &bmp, 0.0, 0.0, 1.0, 1.0, None));
    }



    fn convert_screen_coor_to_pixel_corr(rect: [f64;4])-> [i32; 4]{
        let WindowInfo{focused, width, height} = clone_windowinfo();
        
        let mut rt = [0i32; 4];
        rt[0] = ( ( 1.0 + rect[0] ) * width / 2.0) as i32;
        rt[1] = ( ( 1.0 + rect[1] ) * height / 2.0) as i32;
        rt[2] = ( rect[2] * width / 2.0) as i32;
        rt[3] = ( rect[3] * height / 2.0) as i32;
       
        rt
    }

    //Box movement test
    if mouseinfo.lbutton == ButtonStatus::Down && in_rect( mouseinfo.x as i32, mouseinfo.y as i32,  
                                                  convert_screen_coor_to_pixel_corr([state.x as f64, state.y as f64, 1.0, 1.0]) ) {

        let WindowInfo{focused, width, height} = clone_windowinfo();

        if state.box_in_focus == true {
            state.x += 2.0*(mouseinfo.x - state.prev_mouse[0]) as f32 / width as f32;
            state.y += 2.0*(mouseinfo.y - state.prev_mouse[1]) as f32 / height as f32;
        } else {
            state.box_in_focus = true;
        }
        state.prev_mouse = [ mouseinfo.x, mouseinfo.y];
    } 


    gl_drawtexture(&mut renderer, texture.unwrap(), state.x, state.y, None);
    return None;
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

fn gl_drawtexture<T: glium::Surface>(renderer: &mut Renderer<T>, texture: &glium::texture::Texture2d, x: f32, y: f32, perspective: Option<[[f32;4];4]> ){
    let Renderer{display, target, indices, program} = renderer;

    let w = texture.width();
    let h = texture.height();

    let ratio = w as f32 / h as f32;

    let mut _perspective = perspective.unwrap_or(
                          [  [1.0, 0.0, 0.0, 0.0,],
                             [0.0, 1.0, 0.0, 0.0,],
                             [0.0, 0.0, 1.0, 0.0,],
                             [0.0, 0.0, 0.0, 1.0f32]]
                          );
    //TODO
    //need an x-axis and y-axis scaling term
    let     transform = Matrix4::new(1.0, 0.0, 0.0, 0.0,
                                     0.0, 1.0, 0.0, 0.0,
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
        let vertexbuffer = generate_plane(0.0, ratio, 0.0, 1.0, display);

        target.draw(&vertexbuffer, *indices, program, &uniforms, &Default::default()).unwrap();
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









