use crate::res;
use crate::window;

pub async fn window(pw: &window::PlayerWindow) -> adw::AboutWindow {
    adw::AboutWindow::builder()
        .application_name(res::APP_NAME)
        .version(res::APP_VERSION)
        .developer_name("Jacob Zimmermann")
        .license_type(gtk4::License::Gpl20Only)
        .transient_for(pw)
        .build()
}
