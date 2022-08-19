use crate::domain::service::pocket::{IsLoginOutput, ListOutput, PocketService, StartLoginOutput};
use crate::domain::service::send_to_kindle::{SendInput, SendOutput, SendToKindleService};
use crate::state::AppState;
use anyhow::Result;
use shaku::HasProvider;
use tauri::State;

#[tauri::command]
pub async fn is_login(state: State<'_, AppState>) -> Result<IsLoginOutput, String> {
  let mut service: Box<dyn PocketService> = state.module.provide().unwrap();

  service.is_login().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn logout(state: State<'_, AppState>) -> Result<(), String> {
  let mut service: Box<dyn PocketService> = state.module.provide().unwrap();

  service.logout().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn login(state: State<'_, AppState>) -> Result<StartLoginOutput, String> {
  let mut service: Box<dyn PocketService> = state.module.provide().unwrap();

  service.start_login().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list(state: State<'_, AppState>) -> Result<ListOutput, String> {
  let mut service: Box<dyn PocketService> = state.module.provide().unwrap();

  service.list().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn send(input: SendInput, state: State<'_, AppState>) -> Result<SendOutput, String> {
  let mut service: Box<dyn SendToKindleService> = state.module.provide().unwrap();

  service.send(input).await.map_err(|e| e.to_string())
}
