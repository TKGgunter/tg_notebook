



#[cfg(target_os = "linux")]
pub mod dynamic_lib_loading{

use std::ffi::CString; 
use std::ptr;
use std::os::raw::{c_char, c_int, c_void};

//
//This is a lib of dlopen and dlclose using rust
//Comments copied from the following source files
// /usr/include/dlfcn.h
// https://www.unvanquished.net/~modi/code/include/x86_64-linux-gnu/bits/dlfcn.h.html

    /* These are the possible values for the REQUEST argument to `dlinfo'.  */
    enum DL_INFO{
        /* Treat ARG as `lmid_t *'; store namespace ID for HANDLE there.  */
        RTLD_DI_LMID = 1,

        /* Treat ARG as `struct link_map **';
           store the `struct link_map *' for HANDLE there.  */
        RTLD_DI_LINKMAP = 2,

        RTLD_DI_CONFIGADDR = 3,	/* Unsupported, defined by Solaris.  */

        /* Treat ARG as `Dl_serinfo *' (see below), and fill in to describe the
           directories that will be searched for dependencies of this object.
           RTLD_DI_SERINFOSIZE fills in just the `dls_cnt' and `dls_size'
           entries to indicate the size of the buffer that must be passed to
           RTLD_DI_SERINFO to fill in the full information.  */
        RTLD_DI_SERINFO = 4,
        RTLD_DI_SERINFOSIZE = 5,

        /* Treat ARG as `char *', and store there the directory name used to
           expand $ORIGIN in this shared object's dependency file names.  */
        RTLD_DI_ORIGIN = 6,

        RTLD_DI_PROFILENAME = 7,	/* Unsupported, defined by Solaris.  */
        RTLD_DI_PROFILEOUT = 8,	/* Unsupported, defined by Solaris.  */

        /* Treat ARG as `size_t *', and store there the TLS module ID
           of this object's PT_TLS segment, as used in TLS relocations;
           store zero if this object does not define a PT_TLS segment.  */
        RTLD_DI_TLS_MODID = 9,

        /* Treat ARG as `void **', and store there a pointer to the calling
           thread's TLS block corresponding to this object's PT_TLS segment.
           Store a null pointer if this object does not define a PT_TLS
           segment, or if the calling thread has not allocated a block for it.  */
        RTLD_DI_TLS_DATA = 10,

        //RTLD_DI_MAX = 10
    }

/* The MODE argument to `dlopen' contains one of the following: */
    
    pub const RTLD_LAZY         : i32 =   0x00001;        /* Lazy function call binding.  */
    pub const RTLD_NOW          : i32 =   0x00002;        /* Immediate function call binding.  */
    pub const RTLD_BINDING_MASK : i32 =   0x3    ;    /* Mask of binding time value.  */
    pub const RTLD_NOLOAD       : i32 =   0x00004;        /* Do not load the object.  */
    pub const RTLD_DEEPBIND     : i32 =   0x00008;        /* Use deep binding.  */
    /* If the following bit is set in the MODE argument to `dlopen',
     *    the symbols of the loaded object and its dependencies are made
     *       visible as if the object were linked directly into the program.  */
    pub const RTLD_GLOBAL       : i32 =  0x00100;
    /* Unix98 demands the following flag which is the inverse to RTLD_GLOBAL.
     *    The implementation does this by default and so we can define the
     *       value to zero.  */
    pub const RTLD_LOCAL       : i32 = 0;
    /* Do not delete object when closed.  */
    pub const RTLD_NODELETE    : i32 = 0x01000;

    struct Dl_info{
      dli_fname: *mut c_char,	/* File name of defining object.  */
      dli_fbase: *mut c_void,	/* Load address of that object.  */
      dli_sname: *mut c_char,	/* Name of nearest symbol.  */
      dli_saddr: *mut c_void,	/* Exact value of nearest symbol.  */
      //dlerror
    }
    /* This is the type of elements in `Dl_serinfo', below.
       The `dls_name' member points to space in the buffer passed to `dlinfo'.  */
    struct Dl_serpath
    {
      dls_name: *mut c_char,		/* Name of library search path directory.  */
      dls_flags: u32,	/* Indicates where this directory came from. */
    }

    /* This is the structure that must be passed (by reference) to `dlinfo' for
       the RTLD_DI_SERINFO and RTLD_DI_SERINFOSIZE requests.  */
    struct Dl_serinfo
    {
      dls_size: usize,		/* Size in bytes of the whole buffer.  */
      dls_cnt: u32,		/* Number of elements in `dls_serpath'.  */
      dls_serpath: [Dl_serpath;1],	/* Actually longer, dls_cnt elements.  */
    } 

    //TODO
    //Think about changing from c_int to i32 or something
    extern "C" {
        pub fn dlopen(filename: *const c_char, flag: c_int) -> *mut c_void;
        pub fn dlsym(lib_handle: *mut c_void, name: *const c_char) -> *mut c_void;
        pub fn dlclose(lib_handle: *mut c_void) -> c_int;
        pub fn dlinfo(lib_handle: *mut c_void, request: c_int, info: *mut c_void) -> c_int;
        pub fn dlerror() -> *mut c_char;
    }

    pub struct DyLib(*mut c_void);

    pub fn open_lib( lib_path: &str, flag: i32 )->Result<DyLib, String>{unsafe{

        //TODO
        //Get enums dlopen uses
        let shared_lib_handle = dlopen(CString::new(lib_path).unwrap().as_ptr(), flag as c_int);
        if shared_lib_handle.is_null(){
            println!("{:?}", get_error());
            Err(format!("Shared lib is null! {}  Check file path/name.", lib_path))
        }
        else{
            Ok( DyLib(shared_lib_handle) )
        }

    }}

    //Example
    //let function : fn()->i32= transmute_copy((dlsym(shared_lib_handle, CString::new(name).unwrap().as_ptr()) as *mut ()).as_mut());
    pub fn get_fn( shared_lib_handle: &DyLib, name: &str)-> Result<*mut (), String>{ unsafe{
        let _fn = dlsym(shared_lib_handle.0, CString::new(name).unwrap().as_ptr());
        if _fn.is_null() {
           Err("Function name could not be found.".to_string()) 
        }
        else{
            Ok(_fn as *mut () )
        }
    }}

    pub fn get_error()->String{unsafe{
        let error = dlerror();
        if error.is_null(){
            return "No Error".to_string();
        }
        else{
            CString::from_raw(error).into_string().unwrap()
        }
    }}

    pub fn close_lib(shared_lib_handle: &DyLib){unsafe{
        if dlclose(shared_lib_handle.0) != 0{
            println!("Could not properly close shared library.");
        }
    }}
}

#[cfg(target_os = "windows")]
pub mod dynamic_lib_loading{
use std::os::raw::{c_char, c_int, c_void};

    extern "C" {
        fn LoadLibraryA( path: *const i8 ) -> *mut c_void;
        fn GetProcAddress( lib: *mut c_void, name: *const i8 ) -> *mut c_void;
        fn FreeLibrary( lib: *mut c_void ) -> c_int;
        fn GetLastError() -> u32;
    }

    //TODO
    //This is temporary should be replaced by windows enums
    pub const RTLD_LAZY         : i32 =   0x00001;        /* Lazy function call binding.  */

    pub struct DyLib(*mut c_void);

    pub fn open_lib( lib_path: &str, flag: i32 )->Result<DyLib, String>{unsafe{
        let _path = lib_path.to_string() + "\0";
        let lib = LoadLibraryA( _path.as_ptr() as *const i8);
        if lib.is_null(){
            let s = format!("Could not open lib \n{:?}\n\n For more info => https://docs.microsoft.com/en-us/windows/win32/debug/system-error-codes", GetLastError());
            return Err(s);
        }

        Ok(DyLib(lib as *mut c_void))
    }}

    //Example
    //let function : fn()->i32= transmute_copy((dlsym(shared_lib_handle, CString::new(name).unwrap().as_ptr()) as *mut ()).as_mut());
    pub fn get_fn( shared_lib_handle: &DyLib, name: &str)-> Result<*mut (), String>{ unsafe{
        let fn_name = name.to_string() + "\0";
        let function = GetProcAddress(shared_lib_handle.0 as _, fn_name.as_ptr() as *const i8) as *mut ();
        if function.is_null(){
            let s = format!("Could not get function \n{:?}", GetLastError());
            return Err(s);
        }

        Ok(function)
    }}

    pub fn get_error()->String{
        "Windows version has not been implemented".to_string()
    }

    pub fn close_lib(shared_lib_handle: &DyLib){unsafe{
        if FreeLibrary(shared_lib_handle.0 as _) == 0{
            println!("Could not properly close shared library.");
            println!("{}", format!("{:?}", GetLastError()));
        }
    }}
}



pub mod render_tools{

    pub enum RenderType{
        Image,
        Rectangle,
        String,
        Empty,
    }
    impl Default for RenderType{
        fn default()->Self{ RenderType::Empty }
    }

    #[derive(Default)]
    pub struct RenderStruct{
        pub rendertype : RenderType,
        pub x: f32,//TODO FUTURE should these be options  .... the x and y could be decided internally
        pub y: f32,//TODO FUTURE should these be options  .... the x and y could be decided internally
        pub width:  f32,
        pub height: f32,
        pub alpha : f32,

        pub print_string: bool,
         
        //rect related things
        pub filled: bool,
        pub color: [f32;3],
         
        //image related things
        pub color_buffer: Vec<u8>,
        pub rgba_type: RGBA,
        pub new_width:  Option<f32>,// NOTE Testing out using a factional new width 
        pub new_height: Option<f32>,// NOTE Testing out using a factional new height

        //Stings
        pub char_buffer: String,
        pub font_size: u32
    }


    #[derive(Default)]
    pub struct RenderInstructions{
        pub buffer: Vec<RenderStruct>,
    }
    //TODO 
    //This is a BAD name.... do better
    #[derive(Clone, Copy, PartialEq)]
    pub enum RGBA{
        U8rgba,
        U8argb,
        U8rgb,
        Empty,
        //More maybe ... maybe not
    }
    impl RenderInstructions{
        pub fn clear(&mut self){
            self.buffer.clear();
        }
        pub fn draw_rect(&mut self, rect: [f32; 4], color: [f32; 4], filled: bool){
            let _color = [color[0], color[1], color[2]];
            self.buffer.push( RenderStruct{rendertype: RenderType::Rectangle, 
                                    x: rect[0], y:rect[1], width: rect[2], height: rect[3], 
                                    alpha: color[3], filled: filled, color: _color, .. Default::default()});
        }
        pub fn draw_string(&mut self, s: &str, color: [f32; 4], size: u32, x: f32, y: f32 ){
            //TODO
            //should size be optional 
            //shouldn't a good text size be choosen automatically
            //TODO
            //should color be optional 
            //shouldn't a good text color be choosen automatically
            let _color = [color[0], color[1], color[2]];
            self.buffer.push( RenderStruct{ rendertype: RenderType::String, x: x, y: y, 
                                            alpha: color[3], color: _color, char_buffer: s.to_string(), font_size: size, .. Default::default()} ); 
        }

        pub fn draw_bmp(&mut self, bmp: &Bitmap, x: f32, y: f32, alpha: f32, w: Option<f32>, h: Option<f32>){
            //TODO
            //should x and y be options, Often I want to just draw the image where ever and have it
            //automagicly look good with corresponding text
            self.buffer.push( RenderStruct{rendertype: RenderType::Image, alpha: alpha, x: x, y: y, width: bmp.width as f32, height: bmp.height as f32,
                                           new_width: w, new_height: h, rgba_type: bmp.rgba_type, color_buffer: bmp.buffer.clone(), .. Default::default()} ); 
        }
        pub fn println(&mut self, string: &str){
            let buffer = "> ".to_string() + string;
            self.buffer.push( RenderStruct{ rendertype: RenderType::String, print_string: true,
                                            alpha: 1.0, color: [1.0, 1.0, 1.0], char_buffer: buffer, font_size: 19, .. Default::default()} ); 
        }
    }

    impl Default for RGBA{
        fn default()->Self{ RGBA::Empty }
    }


    #[derive(Clone)]
    pub struct Bitmap{
    //NOTE BMP should be 400 x 400 to start off.
        pub width: i32,
        pub height: i32,
        pub rgba_type: RGBA,
        pub buffer: Vec<u8>,
    }
    impl Bitmap{
        pub fn new(w: i32, h: i32, rgba_type: RGBA)->Bitmap{
            let _w = w as usize;
            let _h = h as usize;
            let v = match rgba_type{
                RGBA::U8rgba=>{ vec![0u8; _w*_h*4] },
                RGBA::U8argb=>{ vec![0u8; _w*_h*4] },
                RGBA::U8rgb=>{  vec![0u8; _w*_h*3] },
                _=>{ panic!("Not supported"); } //TODO Clean up
            };
            Bitmap{
                width: w,
                height: h,
                rgba_type: rgba_type,
                buffer: v, 
            }
        }
    }


    pub struct BitmapContainer{
        pub initialized : bool,
        pub bmp: Option<Bitmap>,
    }
}





pub mod memory_tools{
//TODO play around with this maybe
//use std::alloc;
use std::any::TypeId;
const _FIXED_CHAR_BUFFER_SIZE : usize = 128; 


    #[derive(Copy, Clone)]
    pub struct TinyString{
        //NOTE
        //currently struct vars are public for debugging purposed.  They should not be public.
        //NOTE
        //This should prob be a general tool
        pub buffer: [char; _FIXED_CHAR_BUFFER_SIZE],
        pub cursor: usize,
    }
    impl TinyString{
        pub fn new()->TinyString{
            TinyString{
                buffer: ['\0'; _FIXED_CHAR_BUFFER_SIZE],
                cursor: 0,
            }
        }
        pub fn get(&self, index: usize)->&char{
            &self.buffer[index]
        }
        pub fn get_mut(&mut self, index: usize)->&mut char{
            &mut self.buffer[index]
        }
        pub fn len(&self)->usize{
            self.cursor
        }

        pub fn push(&mut self, c: char)->Result< () , String>{
            if self.len() >= _FIXED_CHAR_BUFFER_SIZE { return Err("Out sized tinystring".to_string()); }
            let _i = self.cursor;
            self[_i] = c;
            self.cursor += 1;
            return Ok( () );
        }

        //TODO
        //check meaning of clone and copy in rust
        pub fn copystr(&mut self, s: &str){
            let mut chars = s.chars();
            for _ in 0.. _FIXED_CHAR_BUFFER_SIZE {
                match chars.next(){
                    Some(c)=> {
                        let _i = self.cursor;
                        self[_i] = c;
                        self.cursor += 1;
                    }
                    _=> break
                }
            }
        }
        //TODO
        //check meaning of clone and copy in rust
        pub fn copy(&mut self, s: &TinyString){
            for i in 0..s.len(){
                self[i] = s[i];
            }
        }

        pub fn is_samestr(&self, s: &str)->bool{
            let mut chars = s.chars();
            if self.len() !=  s.len(){ return false; }
            for i in 0..self.len(){
                if self[i] != chars.next().unwrap(){ return false; }
            }
            return true;
        }

        pub fn is_same(&self, s: &TinyString)->bool{
            if self.len() !=  s.len(){ return false; }
            for i in 0..self.len(){
                if self[i] != s[i]{ return false; }
            }
            return true;
        }
    }
    impl std::ops::Index<usize> for TinyString{
        type Output = char;
        fn index(&self, index: usize)->&Self::Output{
            self.get(index)
        }
    }
    impl std::ops::IndexMut<usize> for TinyString{
        fn index_mut(&mut self, index: usize)->&mut Self::Output{
            self.get_mut(index)
        }
    }

    pub trait Storage{
        fn get_storage(&mut self)->&mut [u8];
    }
    impl Storage for GlobalStorage{
        fn get_storage(&mut self)->&mut [u8]{
            &mut self.storage
        }
    }





    pub struct Ptr<S: Storage>{
        //TODO make backend_storage a Generic function that requires trait
        ptr: usize,
        type_hash: TypeId,
        backend_storage: *mut S
    }
    impl<S: Storage> Ptr<S>{
        pub fn deref_mut<T: 'static>(&self)->&mut T{unsafe{
            if self.type_hash != TypeId::of::<T>() { panic!("Could not dereference custom pointer do to failed hash comparison.");}
            let gs = self.backend_storage.as_mut().unwrap();

            ((&mut gs.get_storage()[self.ptr] as *mut u8 ) as *mut T).as_mut().unwrap()
        }}
    }


    impl<S: Storage> std::clone::Clone for Ptr<S>{

        fn clone(&self)->Self{
            Ptr{ 
                ptr: self.ptr,
                type_hash: self.type_hash,
                backend_storage: self.backend_storage,
            }
        }
    }

    pub struct DyArray<T>{
        ptr: Ptr<GlobalStorage>,
        length: usize,
        capacity: usize,

        phantom: std::marker::PhantomData<T>,
    }
    impl<T: 'static> DyArray<T>{
        pub fn push(&mut self, v: T) {unsafe{
            let gs = self.ptr.backend_storage.as_mut().unwrap();
            if self.length >= self.capacity{
                let length = self.length * std::mem::size_of::<T>();
                let old_ptr = self.ptr.clone();
                let new_ptr = gs.realloc::<T>(old_ptr, length, self.capacity);
                self.ptr = new_ptr;
                return;
            }
            {
                let cursor = self.ptr.ptr + self.length * std::mem::size_of::<T>();
                gs.write_to(v, cursor);

                self.length +=1;
            }
        }}

        pub fn new(gs: &mut GlobalStorage)->DyArray<T>{
            DyArray::<T>::with_capacity(gs, 5)
        }
        pub fn with_capacity(gs: &mut GlobalStorage, size: usize)->DyArray<T>{
            let ptr = gs.alloc_multi_empty::<T>( size );
            DyArray{
                ptr: ptr,
                length: 0,
                capacity: size,

                phantom: std::marker::PhantomData
            }
        }

        pub fn get(&self, index: usize)->&T{unsafe{
            if index > self.length {
                panic!("Index bounds error.");
            }

            let base = self.ptr.ptr;
            let address = base + index * std::mem::size_of::<T>();

            let gs = self.ptr.backend_storage.as_mut().unwrap();
            ((&mut gs.get_storage()[address] as *mut u8 ) as *mut T).as_mut().unwrap()
        }}

        pub fn get_mut(&mut self, index: usize)->&mut T{unsafe{
            if index > self.length {
                panic!("Index bounds error.");
            }
            let base = self.ptr.ptr;
            let address = base + index * std::mem::size_of::<T>();

            let gs = self.ptr.backend_storage.as_mut().unwrap();
            ((&mut gs.get_storage()[address] as *mut u8 ) as *mut T).as_mut().unwrap()
        }}
    }


    //TODO 
    //Convert Global and Local storage into a general storage storage thing where storage can be
    //either a fixed or dynamic array.
    pub struct GlobalStorage{
        pub storage: Vec<u8>, 
        pub storage_filled: Vec<bool>,      //TODO speace footprint improvement use bits in u8
        reference: [TinyString;100],    //This is fixed size because I really want to stop myself from over populating the global space
        stored_ptr: Vec<Ptr<GlobalStorage>>,  
    }

    impl GlobalStorage{
        pub fn new()->GlobalStorage{
            GlobalStorage{
                storage: Vec::with_capacity(1028*1028*4),           //This is still prob too small
                storage_filled: Vec::with_capacity(1028*1028*4),   

                //TODO
                //reference needs to store the ptr index TODO
                reference: [TinyString::new(); 100],
                stored_ptr: Vec::new(),  
            }
        }
        pub fn alloc<T: 'static>(&mut self, v: T)->Ptr<GlobalStorage>{unsafe{
            let size = std::mem::size_of::<T>();
            let src = (&v as *const T) as *const u8;

            let cursor = self.storage.len();
            for i in 0..size{
               //TODO
               //SLOW SPEED ME UP
               //I don't think I want to be pushing every thiem like this
               //TODO 
               //byte alignments
               self.storage.push(*src.offset(i as isize));
               self.storage_filled.push(true);
            }
            return Ptr{ ptr: cursor, type_hash: TypeId::of::<T>(), backend_storage: self as *mut _};
        }}
        pub fn alloc_multi_empty<T: 'static>(&mut self, multiples: usize)->Ptr<GlobalStorage>{
            let size = std::mem::size_of::<T>() * multiples;

            let cursor = self.storage.len();
            for i in 0..size{
               //TODO
               //SLOW SPEED ME UP
               //I don't think I want to be pushing every thiem like this
               //TODO 
               //byte alignments
               self.storage.push(0);
               self.storage_filled.push(true);
            }
            return Ptr{ ptr: cursor, type_hash: TypeId::of::<T>(), backend_storage: self as *mut _};
        }
        pub fn realloc<T>(&mut self, ptr: Ptr<GlobalStorage>, index_back: usize, additional_space: usize)->Ptr<GlobalStorage>{
            //TODO
            //SLOW SPEED UP
            let cursor = self.storage.len();

            let index_front = ptr.ptr;
            for i in index_front..index_back{
                let temp = self.storage[i];
                self.storage_filled[i] = false;

                self.storage.push(temp);
                self.storage_filled.push(true);
            }
            for i in 0..additional_space{
                self.storage.push(0);
                self.storage_filled.push(true);
            }
            return Ptr{ ptr: cursor, type_hash: ptr.type_hash, backend_storage: self as *mut _};
        }

        pub unsafe fn write_to<T: 'static>(&mut self, v: T, at_index: usize)->Result<(),String>{
            let size = std::mem::size_of::<T>();
            let src = (&v as *const T) as *const u8;

            if at_index >= self.storage.len() {
                return Err("Writing outside the bounds of memory allocated to global storage".to_string());
            }

            let cursor = at_index;
            for i in 0..size{
               //TODO
               //SLOW SPEED ME UP
               //I don't think I want to be pushing every thiem like this
               //TODO 
               //byte alignments
               if !self.storage_filled[cursor+i]  { panic!("Storage has not allocated this memory.") }
               self.storage[cursor+i] = *src.offset(i as isize);
            }
            return Ok(());
        }
        pub fn store<T: 'static>(&mut self, v: T, name: &str)->Result<(), String>{
            if name.len() > _FIXED_CHAR_BUFFER_SIZE {
                return Err("storage name is too damn long".to_string());
            }

            let cursor = self.stored_ptr.len();
            for it in self.reference.iter() {
                if it.is_samestr(name){
                    return Err(format!("Global Storage name collision: {}", name));        
                }
            }
            self.reference[cursor].copystr( name );
            let ptr = self.alloc(v);
            self.stored_ptr.push(ptr);
            return Ok(());
        }

        pub fn get<T: 'static>(&mut self, name: &str)->Result<&mut T, String>{
            let cursor = self.stored_ptr.len();
            let mut isgood = false;
            let mut ptr_index = 0;
            for (i, it) in self.reference.iter().enumerate() {
                if it.is_samestr(name){
                    ptr_index = i;
                    isgood = true;
                }
            }
            if isgood == false { return Err(format!("Name not found in Global Storage: {}", name)); }
            let ptr = &self.stored_ptr[ptr_index];
            return Ok(ptr.deref_mut::<T>());
        }
        //

    }

    pub struct LocalStorage{
        //NOTE
        //This seems to be a good idea when it comes to interactive panels
        //However I'm not sure about the usefulness else where....
        //
        //
        //Why should the local buffer be fixed sized. This doesn't really make sense.
        pub initialized: bool,
    }
    impl LocalStorage{
        pub fn new()->LocalStorage{
            LocalStorage{
                initialized: false,
            }
        }
    }

}

pub mod interaction_tools{
    pub enum KeyboardEnum{
        Rightarrow,
        Leftarrow,
        Uparrow,
        Downarrow,
        Enter,
        Default
    }
    impl Default for KeyboardEnum{
        fn default()->Self{ KeyboardEnum::Default }
    }


    pub enum ButtonStatus{
        Up,
        Down,
        Default
    }
    impl Default for ButtonStatus{
        fn default()->Self{ ButtonStatus::Default }
    }

    #[derive(Default)]
    pub struct InteractiveInfo{
        pub infocus: bool,

        pub mouse_x: f32,
        pub mouse_y: f32,

        pub text_key_pressed: char,
        pub keyboard_key: Vec<KeyboardEnum>,
        pub keyboard_key_status: Vec<ButtonStatus>,
    }

}



#[test]
fn globalstorage_alloc_and_store(){
    //TODO rename

    use memory_tools::GlobalStorage;

    let mut gls = GlobalStorage::new();

    {
        let mut a = [10u8; 4];
        gls.store(a, "a");
    }
    let mut b = [10u8; 4];
    assert_eq!(b, *gls.get::<[u8;4]>("a").unwrap());
}    


#[test]
fn globalstorage_vec(){
    //TODO rename
    use memory_tools::{GlobalStorage, DyArray};
    let mut gls = GlobalStorage::new();
    let mut dy = DyArray::<u32>::new(&mut gls);

    dy.push(12);
    dy.push(123);
    dy.push(1231);

    //let a = dy.get(0);
    //assert_eq!(12, *a);

    println!("print test");
    let a = dy.get(1);
    assert_eq!(123, *a);

    let a = dy.get(2);
    assert_eq!(1231, *a);
     
    /* Does not compile for some reason
     * I need to inform the rust folks
    assert_eq!(12,   *(dy.get(0)));
    assert_eq!(123,  *(dy,get(1)));
    assert_eq!(1231, *(dy.get(2)));
   */ 
}


#[test]
fn global_storage_vec2(){
    //TODO rename
    use memory_tools::{GlobalStorage, DyArray};
    let mut gls = GlobalStorage::new();
    let mut a  = gls.alloc(123.09f32);
    let mut dy = DyArray::<u32>::new(&mut gls);
    let mut b  = gls.alloc(123i32);

    dy.push(12);
    dy.push(123);
    dy.push(1231);

    //let a = dy.get(0);
    //assert_eq!(12, *a);

    let ab = dy.get(1);
    assert_eq!(123, *ab);

    let ab = dy.get(2);
    assert_eq!(1231, *ab);
    
}


#[test]
fn global_storage_vec3(){
    //TODO rename
    use memory_tools::{GlobalStorage, DyArray};
    let mut gls = GlobalStorage::new();
    {
        let mut dy = DyArray::<u32>::new(&mut gls);

        dy.push(12);
        dy.push(123);
        dy.push(1231);

        gls.store(dy, "dy");
    }
    
    let a = gls.get::<DyArray<u32>>("dy").unwrap();
    let ab = a.get(0);
    assert_eq!(12, *ab);
    
}



