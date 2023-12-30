use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk4::{self, gio, glib};

use crate::{res, window};

mod private {
    use super::*;

    #[derive(Default)]
    pub struct Gemini;

    #[glib::object_subclass]
    impl ObjectSubclass for Gemini {
        const NAME: &'static str = "Gemini";
        type Type = super::Gemini;
        type ParentType = adw::Application;
    }

    impl ObjectImpl for Gemini {
        fn constructed(&self) {
            //let obj=self.obj();
            self.parent_constructed();
        }
    }

    impl ApplicationImpl for Gemini {
        fn activate(&self) {
            let obj = self.obj();
            let app = obj.as_ref();
            let pw = window::PlayerWindow::builder().application(app).build();
            app.add_window(&pw);
            pw.present();
        }

        fn open(&self, files: &[gtk4::gio::File], hint: &str) {
            //let obj=self.obj();
        }
    }

    impl GtkApplicationImpl for Gemini {}
    impl AdwApplicationImpl for Gemini {}
}

glib::wrapper! {
    pub struct Gemini(ObjectSubclass<private::Gemini>)
        @extends gio::Application, gtk4::Application, adw::Application,
        @implements gio::ActionGroup, gio::ActionMap;
}

pub struct GeminiBuilder;

impl GeminiBuilder {
    fn new() -> Self {
        Self
    }

    pub fn build(self) -> Result<Gemini, glib::Error> {
        let gres = {
            let gb = glib::Bytes::from_static(res::RESOURCES);
            gio::Resource::from_data(&gb)
        }?;
        gio::resources_register(&gres);

        let app = glib::Object::builder()
            .property("application-id", res::APP_ID)
            .property("flags", gio::ApplicationFlags::HANDLES_OPEN)
            .property("resource-base-path", res::APP_BASE_PATH)
            .build();
        Ok(app)
    }
}

impl Gemini {
    pub fn builder() -> GeminiBuilder {
        GeminiBuilder::new()
    }
}
