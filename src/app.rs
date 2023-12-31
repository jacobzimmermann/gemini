use adw::prelude::*;
use std::cell;

use adw::subclass::prelude::*;
use gtk4::{self, gio, glib};
use log;

use crate::{about, res, window};

mod private {
    use super::*;

    #[derive(Default)]
    pub struct Gemini {
        window_id: cell::Cell<u32>,
    }

    impl Gemini {
        pub(super) fn window_id(&self) -> u32 {
            self.window_id.get()
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Gemini {
        const NAME: &'static str = "Gemini";
        type Type = super::Gemini;
        type ParentType = adw::Application;
    }

    impl ObjectImpl for Gemini {
        fn constructed(&self) {
            self.parent_constructed();

            let app = self.obj();
            app.create_actions();
        }
    }

    impl ApplicationImpl for Gemini {
        fn activate(&self) {
            let obj = self.obj();
            let app = obj.as_ref();
            let pw = window::PlayerWindow::builder().application(app).build();
            app.add_window(&pw);
            pw.present();

            self.window_id.set(pw.id());
        }

        fn open(&self, files: &[gtk4::gio::File], _hint: &str) {
            //let obj=self.obj();
            log::debug!("Dropped {files:?}");
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

    fn window(&self) -> window::PlayerWindow {
        let win_id = self.imp().window_id();
        let win = self.window_by_id(win_id).unwrap();
        win.downcast::<window::PlayerWindow>().unwrap()
    }

    fn create_actions(&self) {
        let actions = [gio::ActionEntryBuilder::new("about")
            .activate(|app: &Self, _, _| {
                let app = app.clone();
                glib::spawn_future_local(async move { app.show_about().await });
            })
            .build()];

        self.add_action_entries(actions);
    }

    async fn show_about(&self) {
        let pw = self.window();
        let aw = about::window(&pw).await;
        aw.present();
    }
}
