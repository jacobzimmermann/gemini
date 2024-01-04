use std::cell::{Cell, OnceCell, RefCell};
use std::time::Duration;

use adw::prelude::*;
use adw::subclass::prelude::*;
use glib_macros::clone;
use gst::prelude::*;
use gtk4::{gdk, gio, glib};

mod private {
    use super::*;

    #[derive(Default, gtk4::CompositeTemplate, glib::Properties)]
    #[properties(wrapper_type = super::PlayerWindow)]
    #[template(resource = "/player-window.ui")]
    pub struct PlayerWindow {
        #[property(get, set)]
        uri: RefCell<glib::GString>,

        #[property(get, set)]
        fullscreen: Cell<bool>,

        player: OnceCell<gstplay::Play>,

        #[template_child]
        drop_target: gtk4::TemplateChild<gtk4::DropTarget>,

        #[template_child]
        video_area: gtk4::TemplateChild<gtk4::Overlay>,

        #[template_child]
        video_widget: gtk4::TemplateChild<gtk4::Picture>,

        #[template_child]
        fullscreen_button: gtk4::TemplateChild<gtk4::Button>,

        #[template_child]
        clock_label: gtk4::TemplateChild<gtk4::Label>,
    }

    impl PlayerWindow {
        pub(super) fn get_player(&self) -> &gstplay::Play {
            self.player.get_or_init(|| {
                let g4sink = gst::ElementFactory::make("gtk4paintablesink")
                    .build()
                    .expect("Failed to create video sink");
                let paintable = g4sink.property::<gdk::Paintable>("paintable");
                self.video_widget.set_paintable(Some(&paintable));

                let videosink = if paintable
                    .property::<Option<gdk::GLContext>>("gl-context")
                    .is_some()
                {
                    log::info!("Using GL");
                    gst::ElementFactory::make("glsinkbin")
                        .property("sink", &g4sink)
                        .build()
                        .expect("Failed to create GL sink")
                } else {
                    log::info!("GL is not available");
                    let sink = gst::Bin::default();
                    let convert = gst::ElementFactory::make("videoconvert")
                        .build()
                        .expect("Failed to create video converter");
                    sink.add_many([&convert, &g4sink])
                        .expect("Failed to create video bin");
                    convert
                        .link(&g4sink)
                        .expect("Failed to link converter to video sink");
                    let cpad = convert
                        .static_pad("sink")
                        .expect("Video sink pad not found");
                    let gpad =
                        gst::GhostPad::with_target(&cpad).expect("Failed to create ghost pad");
                    sink.add_pad(&gpad)
                        .expect("Failed to add ghost pad to video bin");
                    sink.upcast()
                };

                let renderer = gstplay::PlayVideoOverlayVideoRenderer::with_sink(&videosink);
                let player = gstplay::Play::new(Some(renderer));

                glib::timeout_add_local(Duration::from_millis(500), {
                    let label = self.clock_label.get();
                    let pipeline = player.pipeline();
                    move || {
                        pipeline
                            .query_position::<gst::ClockTime>()
                            .map(|pos| label.set_text(&format!("{:.0}", pos.display())));
                        glib::ControlFlow::Continue
                    }
                });

                player
            })
        }

        fn setup_dnd(&self) {
            let pw = self.obj();

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

    #[glib::object_subclass]
    impl ObjectSubclass for PlayerWindow {
        const NAME: &'static str = "PlayerWindow";
        type Type = super::PlayerWindow;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);

            klass.install_property_action("win.toggle-fullscreen", "fullscreen");
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template()
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for PlayerWindow {
        fn constructed(&self) {
            self.parent_constructed();

            let player = self.get_player();
            let pw = self.obj();
            pw.bind_property("uri", player, "uri")
                .bidirectional()
                .build();
            pw.connect_uri_notify(super::PlayerWindow::on_uri_change);
            pw.connect_fullscreen_notify(super::PlayerWindow::on_toggle_fullscreen);

            self.setup_dnd();
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
        log::info!("Start playing {s_uri}");
        let player = self.imp().get_player();
        player.set_video_track_enabled(true);
        player.play();
    }

    fn on_toggle_fullscreen(&self) {
        let fs = self.fullscreen();
        if fs {
            log::debug!("Going fullscreen")
        } else {
            log::debug!("Exiting fullscreen")
        };
        self.set_fullscreened(fs)
    }
}
