#![windows_subsystem = "windows"]

mod app_tray_icon;
mod platform;

use anyhow::Result;
use futures::{FutureExt, never::Never, select};
use gpui::{
    App, Application, AsyncApp, Global, Window, WindowBounds, WindowHandle, WindowOptions, black,
    div, prelude::*, px, size,
};
use raw_window_handle::HasWindowHandle;
use smol::channel::Receiver;

use crate::{
    app_tray_icon::app_tray_icon,
    platform::{always_on_top, enable_alpha, set_opacity},
};

#[derive(Default)]
pub struct GlobalEntity {
    hide: bool,
}

impl Global for GlobalEntity {}

pub struct WindowEntity {}

impl Render for WindowEntity {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let hide = cx.global::<GlobalEntity>().hide;
        let opacity = if hide { 255 } else { 0 };
        set_opacity(window.window_handle().unwrap().as_ref(), opacity);

        div().size_full().bg(black()).flex()
    }
}

fn open_window(cx: &mut App) -> Result<WindowHandle<WindowEntity>> {
    cx.open_window(
        WindowOptions {
            window_bounds: Some(WindowBounds::centered(size(px(256.), px(78.)), cx)),
            is_resizable: false,
            is_minimizable: false,
            ..WindowOptions::default()
        },
        |window, cx| {
            enable_alpha(window.window_handle().unwrap().as_ref());
            always_on_top(window.window_handle().unwrap().as_ref());

            cx.new(|_| WindowEntity {})
        },
    )
}

fn open_window_async_app(cx: &mut AsyncApp) -> Result<WindowHandle<WindowEntity>> {
    cx.open_window(
        WindowOptions {
            is_resizable: false,
            is_minimizable: false,
            ..WindowOptions::default()
        },
        |window, cx| {
            window.resize(size(px(256.), px(78.)));
            enable_alpha(window.window_handle().unwrap().as_ref());
            always_on_top(window.window_handle().unwrap().as_ref());

            cx.new(|_| WindowEntity {})
        },
    )
}

async fn event_loop(
    click_rx: Receiver<()>,
    new_window_rx: Receiver<()>,
    cx: &mut AsyncApp,
) -> Result<Never> {
    loop {
        select! {
            _ = click_rx.recv().fuse() => {
                cx.update_global::<GlobalEntity, _>(|global, _app| {
                    global.hide = !global.hide;
                })?;
                cx.refresh()?;
            }
            _ = new_window_rx.recv().fuse() => {
                open_window_async_app(cx)?;
            }
        };
    }
}

fn main() {
    let (click_tx, click_rx) = smol::channel::unbounded::<()>();
    let (new_window_tx, new_window_rx) = smol::channel::unbounded::<()>();
    let _tray_icon = app_tray_icon(click_tx, new_window_tx).unwrap();
    Application::new().run(move |cx| {
        cx.set_global(GlobalEntity::default());
        open_window(cx).unwrap();
        cx.spawn(async move |cx| event_loop(click_rx, new_window_rx, cx).await)
            .detach();
    });
}
