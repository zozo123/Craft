//! Port of `src/config.h`.

pub const DEBUG: bool = false;
pub const FULLSCREEN: bool = false;
pub const WINDOW_WIDTH: i32 = 1024;
pub const WINDOW_HEIGHT: i32 = 768;
pub const VSYNC: bool = true;
pub const SCROLL_THRESHOLD: f32 = 0.1;
pub const MAX_MESSAGES: usize = 4;
pub const DB_PATH: &str = "craft.db";
pub const USE_CACHE: bool = true;
pub const DAY_LENGTH: i32 = 600;
pub const INVERT_MOUSE: bool = false;

pub const SHOW_LIGHTS: bool = true;
pub const SHOW_PLANTS: bool = true;
pub const SHOW_CLOUDS: bool = true;
pub const SHOW_TREES: bool = true;
pub const SHOW_ITEM: bool = true;
pub const SHOW_CROSSHAIRS: bool = true;
pub const SHOW_WIREFRAME: bool = true;
pub const SHOW_INFO_TEXT: bool = true;
pub const SHOW_CHAT_TEXT: bool = true;
pub const SHOW_PLAYER_NAMES: bool = true;

/// Key bindings. The C source mixes character literals with GLFW key codes;
/// the GLFW values are inlined here so the client crate does not have to
/// depend on a GLFW binding just to read a constant.
pub mod keys {
    pub const FORWARD: u32 = b'W' as u32;
    pub const BACKWARD: u32 = b'S' as u32;
    pub const LEFT: u32 = b'A' as u32;
    pub const RIGHT: u32 = b'D' as u32;
    pub const JUMP: u32 = 32; // GLFW_KEY_SPACE
    pub const FLY: u32 = 258; // GLFW_KEY_TAB
    pub const OBSERVE: u32 = b'O' as u32;
    pub const OBSERVE_INSET: u32 = b'P' as u32;
    pub const ITEM_NEXT: u32 = b'E' as u32;
    pub const ITEM_PREV: u32 = b'R' as u32;
    pub const ZOOM: u32 = 340; // GLFW_KEY_LEFT_SHIFT
    pub const ORTHO: u32 = b'F' as u32;
    pub const CHAT: u32 = b't' as u32;
    pub const COMMAND: u32 = b'/' as u32;
    pub const SIGN: u32 = b'`' as u32;
}

pub const CREATE_CHUNK_RADIUS: i32 = 10;
pub const RENDER_CHUNK_RADIUS: i32 = 10;
pub const RENDER_SIGN_RADIUS: i32 = 4;
pub const DELETE_CHUNK_RADIUS: i32 = 14;
pub const CHUNK_SIZE: i32 = 32;
pub const COMMIT_INTERVAL: i32 = 5;
