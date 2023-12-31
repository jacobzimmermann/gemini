use adw::prelude::*;
use adw::subclass::prelude::*;
use glib_macros::clone;
use gtk4::{self, gdk, gio, glib};

mod private {
    use gtk4::glib::StaticType;

    use super::*;

    #[derive(Default, gtk4::CompositeTemplate)]
    #[template(resource = "/net/jzimm/Gemini/gemini.ui")]
    pub struct PlayerWindow {
        #[template_child]
        drop_target: gtk4::TemplateChild<gtk4::DropTarget>,
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
            obj.init_template();
        }
    }

    impl ObjectImpl for PlayerWindow {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();

            self.drop_target.set_types(&[gdk::FileList::static_type()]);
            self.drop_target.connect_accept(
                clone!(@weak obj => @default-return false, move |_,_| {
                    log::info!("accept");
                    true
                }),
            );
            self.drop_target.connect_drop(
                clone!(@weak obj => @default-return false, move |_,value,_,_| {
                    match value.get::<gdk::FileList>() {
                        Ok(flist) => {
                            obj.application().map(|app| {
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
            obj.add_controller(self.drop_target.get());
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
}
