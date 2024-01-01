use std::cell::RefCell;

use adw::prelude::*;
use adw::subclass::prelude::*;
use glib_macros::clone;
use gst::prelude::*;
use gtk4::{gdk, gio, glib};

mod private {
    use super::*;

    #[derive(Default, gtk4::CompositeTemplate, glib::Properties)]
    #[properties(wrapper_type = super::PlayerWindow)]
    #[template(resource = "/gemini.ui")]
    pub struct PlayerWindow {
        #[property(get, set)]
        pub(super) uri: RefCell<glib::GString>,

        pub(super) player: gst::Pipeline,

        #[template_child]
        drop_target: gtk4::TemplateChild<gtk4::DropTarget>,

        #[template_child]
        video_area: gtk4::TemplateChild<gtk4::DrawingArea>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PlayerWindow {
        const NAME: &'static str = "PlayerWindow";
        type Type = super::PlayerWindow;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template()
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for PlayerWindow {
        fn constructed(&self) {
            self.parent_constructed();

            let audiosink = gst::ElementFactory::make("autoaudiosink")
                .build()
                .expect("Failed to create audio sink");
            self.player.add_many([&audiosink]).unwrap();

            let pw = self.obj();

            pw.connect_uri_notify(super::PlayerWindow::on_uri_change);

            self.drop_target.set_types(&[gio::File::static_type()]);
            self.drop_target.connect_accept(
                clone!(@weak pw => @default-return false, move |_, _| {
                    log::info!("accept drop");
                    true
                }),
            );
            self.drop_target.connect_drop(
                clone!(@weak pw => @default-return false, move |_, value, _, _| {
                    match value.get::<gdk::FileList>() {
                        Ok(flist) => {
                            pw.application().map(|app| {
                                app.open(flist.files().as_slice(), "File list");
                            });
                            true
                        },
                        Err(err) => {
                            log::error!("{err}");
                            false
                        }
                    }
                }),
            );

            self.video_area.get().add_controller(self.drop_target.get());
        }
    }

    impl WidgetImpl for PlayerWindow {}
    impl WindowImpl for PlayerWindow {}
    impl ApplicationWindowImpl for PlayerWindow {}
    impl AdwApplicationWindowImpl for PlayerWindow {}
}

glib::wrapper! {
    pub struct PlayerWindow(ObjectSubclass<private::PlayerWindow>)
        @extends gtk4::Widget, gtk4::Window, gtk4::ApplicationWindow, adw::ApplicationWindow,
        @implements gio::ActionMap, gio::ActionGroup, gtk4::Native;
}

pub struct PlayerWindowBuilder<'a, A: glib::IsA<gtk4::Application>> {
    app: Option<&'a A>,
}

impl<'a, A: glib::IsA<gtk4::Application>> PlayerWindowBuilder<'a, A> {
    fn new() -> Self {
        Self { app: None }
    }

    pub fn application(self, app: &'a A) -> Self {
        Self {
            app: Some(app),
            ..self
        }
    }

    pub fn build(self) -> PlayerWindow {
        let mut builder = glib::Object::builder();
        if let Some(app) = self.app {
            builder = builder.property("application", app);
        }
        builder.build()
    }
}

impl PlayerWindow {
    pub fn builder<'a, A: glib::IsA<gtk4::Application>>() -> PlayerWindowBuilder<'a, A> {
        PlayerWindowBuilder::new()
    }

    fn on_uri_change(&self) {
        let s_uri = self.uri();
        log::debug!("Start playing {s_uri}");
    }
}
