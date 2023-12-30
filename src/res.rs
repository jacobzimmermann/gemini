use gvdb_macros;

pub const APP_ID: &'static str = "net.jzimm.gemini";
pub const APP_BASE_PATH: &'static str = "/net/jzimm/Gemini";

pub static RESOURCES: &'static [u8] =
    gvdb_macros::include_gresource_from_dir!("/net/jzimm/Gemini", "res");
