use anyhow::Result;
use smol::channel::Sender;
use tray_icon::{
    Icon, MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent,
    menu::{Menu, MenuEvent, MenuItem},
};

use crate::platform::load_icon_from_exe;

pub fn app_tray_icon(click_tx: Sender<()>, new_window_tx: Sender<()>) -> Result<TrayIcon> {
    let menu = Menu::new();
    let new_window_item = MenuItem::new("New Window", true, None);
    let quit_item = MenuItem::new("Quit", true, None);
    menu.append(&new_window_item).unwrap();
    menu.append(&quit_item).unwrap();
    let new_window_id = new_window_item.id().clone();
    let quit_id = quit_item.id().clone();

    let icon = if cfg!(target_os = "windows") {
        load_icon_from_exe()?
    } else {
        Icon::from_rgba(vec![0; 256 * 4], 16, 16).unwrap()
    };
    let tray_icon = TrayIconBuilder::new()
        .with_tooltip("Click to toggle window visibility")
        .with_icon(icon)
        .with_menu(Box::new(menu))
        .with_menu_on_left_click(false)
        .build()?;

    TrayIconEvent::set_event_handler(Some({
        move |event| {
            if let TrayIconEvent::Click {
                id: _,
                position: _,
                rect: _,
                button,
                button_state,
            } = event
                && button == MouseButton::Left
                && button_state == MouseButtonState::Up
            {
                smol::block_on(async {
                    click_tx.send(()).await.unwrap();
                });
            }
        }
    }));
    MenuEvent::set_event_handler(Some(move |event: MenuEvent| match &event.id {
        id if id == &new_window_id => {
            smol::block_on(async {
                new_window_tx.send(()).await.unwrap();
            });
        }
        id if id == &quit_id => {
            std::process::exit(0);
        }
        _ => {}
    }));
    Ok(tray_icon)
}
