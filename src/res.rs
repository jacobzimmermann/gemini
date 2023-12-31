use gvdb_macros;

pub const APP_NAME: &'static str = "Gemini Player";
pub const APP_VERSION: &'static str = "0.1.0";
pub const APP_SOURCE_URL: &'static str = "https://github.com/jacobzimmermann/gemini";

pub const APP_ID: &'static str = "net.jzimm.gemini";
pub const APP_BASE_PATH: &'static str = "/net/jzimm/Gemini";

pub static RESOURCES: &'static [u8] = gvdb_macros::include_gresource_from_dir!("/", "res");
