#[macro_use]
extern crate glium;
extern crate nalgebra;

use nalgebra::core::Matrix4;
use glium::{glutin, Surface};


//NOTE
//what is the time cost of loading various resolutions of textures all the time : it takes about 2 millis to generate a high res bmp
//To generate a texture and push that texture to the gpu it takes another 2-3 millisecs

//TODO
// + move box   X DONE
// + scrolling  X DONE
// + render within texture






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



struct Renderer<'a>{
    display: &'a glium::Display,
    target:  &'a mut glium::Frame,
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

    let mut event_loop = glutin::EventsLoop::new();
    let monitor = event_loop.get_primary_monitor();
    let glutin::dpi::PhysicalSize{width: monitor_width, height: monitor_height} = monitor.get_dimensions();

    let _window = glutin::WindowBuilder::new()
                    .with_dimensions(glutin::dpi::LogicalSize{width:monitor_width / 2.0, height:monitor_height - 70.0}) //TODO Not robust and 
                    .with_title("tg_notebook");                                                                         //should be different per os and user config

    let mut windowinfo = WindowInfo{focused: true, width: monitor_width / 2.0 , height: monitor_height - 70.0 };

    let context = glutin::ContextBuilder::new();
    let display = glium::Display::new(_window, context, &event_loop).unwrap();



    //NOTE use this to change the cursor we will need when we move and resize
    // https://docs.rs/glutin/0.21.0/glutin/struct.Window.html#method.set_cursor
    display.gl_window().window().set_position(glutin::dpi::LogicalPosition{x:monitor_width/2.0, y:0.0}); 
 

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


fn TEMP_FN( state: &mut TEMP_STATE, mut renderer: Renderer, mouseinfo: &MouseInfo, texture: Option<&mut glium::texture::Texture2d> )->Option<glium::texture::Texture2d>{

    
    
}


fn _2TEMP_FN( state: &mut TEMP_STATE, mut renderer: Renderer, mouseinfo: &MouseInfo, texture: Option<&mut glium::texture::Texture2d> )->Option<glium::texture::Texture2d>{
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


fn _TEMP_FN( state: &mut TEMP_STATE, mut renderer: Renderer, mouseinfo: &MouseInfo, texture: Option<&mut glium::texture::Texture2d> )->Option<glium::texture::Texture2d>{
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
fn gl_drawbmp( renderer: &mut Renderer, bmp: &Bmp, x: f32, y: f32, sw: f32, sh: f32, perspective: Option<[[f32;4];4]> )->glium::texture::Texture2d{
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

fn gl_drawtexture(renderer: &mut Renderer, texture: &glium::texture::Texture2d, x: f32, y: f32, perspective: Option<[[f32;4];4]> ){
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









