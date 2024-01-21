use std::cell::{Cell, OnceCell, RefCell};

use adw::prelude::*;
use adw::subclass::prelude::*;
use glib_macros::clone;
use gst::prelude::*;
use gtk4::{gdk, gio, glib};

mod private {
    use super::*;

    pub(super) enum PlayPauseIcon {
        Paused,
        Playing,
    }

    impl Into<&'static str> for PlayPauseIcon {
        fn into(self) -> &'static str {
            use PlayPauseIcon::*;
            match self {
                Paused => "media-playback-start-symbolic",
                Playing => "media-playback-pause-symbolic",
            }
        }
    }

    #[derive(Default, gtk4::CompositeTemplate, glib::Properties)]
    #[properties(wrapper_type = super::PlayerWindow)]
    #[template(resource = "/player-window.ui")]
    pub struct PlayerWindow {
        #[property(get, set)]
        uri: RefCell<glib::GString>,

        #[property(get, set)]
        fullscreen: Cell<bool>,

        #[property(get, set)]
        playing: Cell<bool>,

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

        #[template_child]
        previous_button: gtk4::TemplateChild<gtk4::Button>,

        #[template_child]
        next_button: gtk4::TemplateChild<gtk4::Button>,

        #[template_child]
        play_pause_button: gtk4::TemplateChild<gtk4::Button>,

        #[template_child]
        pub(super) video_slider: gtk4::TemplateChild<gtk4::Scale>,
    }

    impl PlayerWindow {
        pub(super) fn get_player(&self) -> &gstplay::Play {
            self.player.get_or_init(|| {
                let g4sink = gst::ElementFactory::make("gtk4paintablesink")
                    .build()
                    .expect("The video sink creation must succeed");
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
                        .expect("GL sink creation must succeed")
                } else {
                    log::info!("GL is not available");
                    let sink = gst::Bin::default();
                    let convert = gst::ElementFactory::make("videoconvert")
                        .build()
                        .expect("Video converter creation must succeed");
                    sink.add_many([&convert, &g4sink])
                        .expect("Video bin creation must succeed");
                    convert
                        .link(&g4sink)
                        .expect("Converter must link to video sink");
                    let cpad = convert
                        .static_pad("sink")
                        .expect("Video sink pad must exist");
                    let gpad =
                        gst::GhostPad::with_target(&cpad).expect("Ghost pad creation must succeed");
                    sink.add_pad(&gpad)
                        .expect("Ghost pad must be added to video bin");
                    sink.upcast()
                };

                let renderer = gstplay::PlayVideoOverlayVideoRenderer::with_sink(&videosink);
                gstplay::Play::new(Some(renderer))
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

        fn setup_player_message_bus(&self) {
            let player = self.get_player();
            let pw = self.obj();
            let mut pstream = player
                .message_bus()
                .stream_filtered(&[gst::MessageType::Application]);

            glib::spawn_future_local(clone!(@weak pw => async move {
                use futures_util::stream::StreamExt;
                while let Some(msg) = pstream.next().await {
                    use gst::MessageView::*;
                    use gstplay::PlayMessage;
                    match msg.view() {
                        Application(_) => match gstplay::PlayMessage::parse(&msg).unwrap() {
                            PlayMessage::DurationChanged { duration } => pw.on_duration_updated(&duration),
                            PlayMessage::PositionUpdated { position } => pw.on_position_updated(&position),
                            _ => (),
                        },
                        _ => unreachable!(),
                    }
                }
            }));
        }

        pub(super) fn enable_controls(&self, enabled: bool) {
            for but in [
                self.next_button.get().upcast::<gtk4::Widget>(),
                self.previous_button.get().upcast(),
                self.play_pause_button.get().upcast(),
                self.video_slider.get().upcast(),
            ] {
                but.set_sensitive(enabled);
            }
        }

        pub(super) fn set_pause_play_icon(&self, icon: PlayPauseIcon) {
            self.play_pause_button.set_icon_name(icon.into());
        }

        pub(super) fn update_clock_label(&self, clocktime: &gst::ClockTime) {
            self.clock_label
                .set_text(&format!("{:.0}", clocktime.display()));
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
            klass.install_action("win.play", None, |obj, _, _| {
                obj.set_playing(true);
            });
            klass.install_action("win.stop", None, |obj, _, _| {
                obj.set_playing(false);
            });
            klass.install_action("win.previous", None, |obj, _, _| {
                obj.on_previous();
            });
            klass.install_action("win.next", None, |obj, _, _| {
                obj.on_next();
            });
            klass.install_property_action("win.toggle-playing", "playing");
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
            pw.connect_playing_notify(super::PlayerWindow::on_toggle_playing);

            self.setup_dnd();
            self.enable_controls(false);
            self.setup_player_message_bus();

            self.video_slider.connect_change_value(
                clone!(@weak pw => @default-return glib::Propagation::Stop, move |_,_,val| {
                        pw.on_slider_change_value(val);
                        glib::Propagation::Proceed }),
            );
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
        self.set_playing(false);
        self.imp().enable_controls(false);
        let s_uri = self.uri();
        log::info!("Start playing {s_uri}");
        let player = self.imp().get_player();
        player.set_video_track_enabled(true);
        self.set_playing(true);
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

    fn on_toggle_playing(&self) {
        use private::PlayPauseIcon;

        if self.playing() {
            log::debug!("Play");
            let imp = self.imp();
            let player = imp.get_player();
            player.play();

            /*if player.media_info().is_some() {
                imp.start_updating_label();
                imp.set_pause_play_icon(PlayPauseIcon::Playing);
                imp.enable_controls(true);
            } else {
                self.set_playing(false);
                imp.enable_controls(false);
            }*/
            imp.set_pause_play_icon(PlayPauseIcon::Playing);
            imp.enable_controls(true);
        } else {
            log::debug!("Stop");
            let imp = self.imp();
            imp.get_player().pause();
            imp.set_pause_play_icon(PlayPauseIcon::Paused);
        }
    }

    fn on_previous(&self) {
        log::debug!("go to previous chapter");
    }

    fn on_next(&self) {
        log::debug!("go to next chapter");
    }

    fn on_slider_change_value(&self, val: f64) {
        let pos = gst::ClockTime::from_seconds_f64(val);
        let player = self.imp().get_player();
        self.imp().update_clock_label(&pos);
        player.seek(pos);
    }

    fn on_position_updated(&self, position: &Option<gst::ClockTime>) {
        if let Some(p) = position {
            self.imp().video_slider.set_value(p.seconds_f64());
            self.imp().update_clock_label(p);
        }
    }

    fn on_duration_updated(&self, duration: &Option<gst::ClockTime>) {
        let secs = duration.map(gst::ClockTime::seconds_f64).unwrap_or(0.0);
        self.imp().video_slider.set_range(0.0, secs);
    }
}
