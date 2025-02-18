#![cfg(all(unix, not(target_os = "macos")))]

use crate::connection::ConnectionOps;
use crate::os::wayland::connection::WaylandConnection;
use crate::os::wayland::window::WaylandWindow;
use crate::os::x11::connection::XConnection;
use crate::os::x11::window::XWindow;
use crate::{Clipboard, Dimensions, MouseCursor, ScreenPoint, WindowEventReceiver, WindowOps};
use async_trait::async_trait;
use config::ConfigHandle;
use promise::*;
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use std::any::Any;
use std::rc::Rc;

pub enum Connection {
    X11(Rc<XConnection>),
    Wayland(Rc<WaylandConnection>),
}

#[derive(Clone)]
pub enum Window {
    X11(XWindow),
    Wayland(WaylandWindow),
}

impl Connection {
    pub(crate) fn create_new() -> anyhow::Result<Connection> {
        if config::configuration().enable_wayland {
            match WaylandConnection::create_new() {
                Ok(w) => {
                    log::debug!("Using wayland connection!");
                    return Ok(Connection::Wayland(Rc::new(w)));
                }
                Err(e) => {
                    log::debug!("Failed to init wayland: {}", e);
                }
            }
        }
        Ok(Connection::X11(Rc::new(XConnection::create_new()?)))
    }

    pub async fn new_window(
        &self,
        class_name: &str,
        name: &str,
        width: usize,
        height: usize,
        config: Option<&ConfigHandle>,
    ) -> anyhow::Result<(Window, WindowEventReceiver)> {
        match self {
            Self::X11(_) => XWindow::new_window(class_name, name, width, height, config).await,
            Self::Wayland(_) => {
                WaylandWindow::new_window(class_name, name, width, height, config).await
            }
        }
    }

    pub(crate) fn x11(&self) -> Rc<XConnection> {
        match self {
            Self::X11(x) => Rc::clone(x),
            _ => panic!("attempted to get x11 reference on non-x11 connection"),
        }
    }

    pub(crate) fn wayland(&self) -> Rc<WaylandConnection> {
        match self {
            Self::Wayland(w) => Rc::clone(w),
            _ => panic!("attempted to get wayland reference on non-wayland connection"),
        }
    }
}

impl ConnectionOps for Connection {
    fn terminate_message_loop(&self) {
        match self {
            Self::X11(x) => x.terminate_message_loop(),
            Self::Wayland(w) => w.terminate_message_loop(),
        }
    }

    fn run_message_loop(&self) -> anyhow::Result<()> {
        match self {
            Self::X11(x) => x.run_message_loop(),
            Self::Wayland(w) => w.run_message_loop(),
        }
    }
}

impl Window {
    pub async fn new_window(
        class_name: &str,
        name: &str,
        width: usize,
        height: usize,
        config: Option<&ConfigHandle>,
    ) -> anyhow::Result<(Window, WindowEventReceiver)> {
        Connection::get()
            .unwrap()
            .new_window(class_name, name, width, height, config)
            .await
    }
}

unsafe impl HasRawWindowHandle for Window {
    fn raw_window_handle(&self) -> RawWindowHandle {
        match self {
            Self::X11(x) => x.raw_window_handle(),
            Self::Wayland(w) => w.raw_window_handle(),
        }
    }
}

#[async_trait(?Send)]
impl WindowOps for Window {
    async fn enable_opengl(&self) -> anyhow::Result<Rc<glium::backend::Context>> {
        match self {
            Self::X11(x) => x.enable_opengl().await,
            Self::Wayland(w) => w.enable_opengl().await,
        }
    }

    fn finish_frame(&self, frame: glium::Frame) -> anyhow::Result<()> {
        match self {
            Self::X11(x) => x.finish_frame(frame),
            Self::Wayland(w) => w.finish_frame(frame),
        }
    }

    fn close(&self) -> Future<()> {
        match self {
            Self::X11(x) => x.close(),
            Self::Wayland(w) => w.close(),
        }
    }
    fn notify<T: Any + Send + Sync>(&self, t: T)
    where
        Self: Sized,
    {
        match self {
            Self::X11(x) => x.notify(t),
            Self::Wayland(w) => w.notify(t),
        }
    }

    fn hide(&self) -> Future<()> {
        match self {
            Self::X11(x) => x.hide(),
            Self::Wayland(w) => w.hide(),
        }
    }

    fn toggle_fullscreen(&self) -> Future<()> {
        match self {
            Self::X11(x) => x.toggle_fullscreen(),
            Self::Wayland(w) => w.toggle_fullscreen(),
        }
    }

    fn config_did_change(&self, config: &ConfigHandle) -> Future<()> {
        match self {
            Self::X11(x) => x.config_did_change(config),
            Self::Wayland(w) => w.config_did_change(config),
        }
    }

    fn show(&self) -> Future<()> {
        match self {
            Self::X11(x) => x.show(),
            Self::Wayland(w) => w.show(),
        }
    }

    fn set_cursor(&self, cursor: Option<MouseCursor>) -> Future<()> {
        match self {
            Self::X11(x) => x.set_cursor(cursor),
            Self::Wayland(w) => w.set_cursor(cursor),
        }
    }

    fn invalidate(&self) -> Future<()> {
        match self {
            Self::X11(x) => x.invalidate(),
            Self::Wayland(w) => w.invalidate(),
        }
    }

    fn set_title(&self, title: &str) -> Future<()> {
        match self {
            Self::X11(x) => x.set_title(title),
            Self::Wayland(w) => w.set_title(title),
        }
    }

    fn set_icon(&self, image: crate::bitmaps::Image) -> Future<()> {
        match self {
            Self::X11(x) => x.set_icon(image),
            Self::Wayland(w) => w.set_icon(image),
        }
    }

    fn set_inner_size(&self, width: usize, height: usize) -> Future<Dimensions> {
        match self {
            Self::X11(x) => x.set_inner_size(width, height),
            Self::Wayland(w) => w.set_inner_size(width, height),
        }
    }

    fn set_window_position(&self, coords: ScreenPoint) -> Future<()> {
        match self {
            Self::X11(x) => x.set_window_position(coords),
            Self::Wayland(w) => w.set_window_position(coords),
        }
    }

    fn get_clipboard(&self, clipboard: Clipboard) -> Future<String> {
        match self {
            Self::X11(x) => x.get_clipboard(clipboard),
            Self::Wayland(w) => w.get_clipboard(clipboard),
        }
    }
    fn set_clipboard(&self, clipboard: Clipboard, text: String) -> Future<()> {
        match self {
            Self::X11(x) => x.set_clipboard(clipboard, text),
            Self::Wayland(w) => w.set_clipboard(clipboard, text),
        }
    }
}
