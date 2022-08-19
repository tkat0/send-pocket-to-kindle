#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use dotenv::dotenv;
use state::AppState;
use tauri::api::shell;
use tauri::{
  AboutMetadata, CustomMenuItem, Manager, Menu, MenuEntry, MenuItem, State, Submenu, WindowBuilder,
  WindowUrl,
};

mod command;
mod domain;
mod env;
mod library;
mod repository;
mod service;
mod state;

fn main() {
  let ctx = tauri::generate_context!();
  dotenv().ok();

  env_logger::init();

  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![
      command::is_login,
      command::login,
      command::logout,
      command::list,
      command::send,
    ])
    .setup(|app| {
      let _window = WindowBuilder::new(app, "main", WindowUrl::default())
        .title("Pocket to Kindle")
        .inner_size(800.0, 550.0)
        .min_inner_size(400.0, 200.0)
        .build()
        .expect("Unable to create window");

      app.manage(AppState::new(&app)?);

      Ok(())
    })
    .menu(Menu::with_items([
      #[cfg(target_os = "macos")]
      MenuEntry::Submenu(Submenu::new(
        &ctx.package_info().name,
        Menu::with_items([
          MenuItem::About(ctx.package_info().name.clone(), AboutMetadata::default()).into(),
          MenuItem::Separator.into(),
          MenuItem::Services.into(),
          MenuItem::Separator.into(),
          MenuItem::Hide.into(),
          MenuItem::HideOthers.into(),
          MenuItem::ShowAll.into(),
          MenuItem::Separator.into(),
          MenuItem::Quit.into(),
        ]),
      )),
      MenuEntry::Submenu(Submenu::new(
        "File",
        Menu::with_items([MenuItem::CloseWindow.into()]),
      )),
      MenuEntry::Submenu(Submenu::new(
        "Edit",
        Menu::with_items([
          MenuItem::Undo.into(),
          MenuItem::Redo.into(),
          MenuItem::Separator.into(),
          MenuItem::Cut.into(),
          MenuItem::Copy.into(),
          MenuItem::Paste.into(),
          #[cfg(not(target_os = "macos"))]
          MenuItem::Separator.into(),
          MenuItem::SelectAll.into(),
        ]),
      )),
      MenuEntry::Submenu(Submenu::new(
        "View",
        Menu::with_items([MenuItem::EnterFullScreen.into()]),
      )),
      MenuEntry::Submenu(Submenu::new(
        "Window",
        Menu::with_items([MenuItem::Minimize.into(), MenuItem::Zoom.into()]),
      )),
      // You should always have a Help menu on macOS because it will automatically
      // show a menu search field
      MenuEntry::Submenu(Submenu::new(
        "Help",
        Menu::with_items([CustomMenuItem::new("Learn More", "Learn More").into()]),
      )),
    ]))
    .on_menu_event(|event| {
      let event_name = event.menu_item_id();
      match event_name {
        "Learn More" => {
          let url = "https://github.com/tkat0/send-pocket-to-kindle".to_string();
          shell::open(&event.window().shell_scope(), url, None).unwrap();
        }
        _ => {}
      }
    })
    .run(ctx)
    .expect("error while running tauri application");
}
