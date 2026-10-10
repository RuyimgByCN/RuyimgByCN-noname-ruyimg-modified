use content_runtime::{ActiveCatalogPage, InstallResult, PackageSignature};
use desktop_client::{ContentBundle, DesktopHost, PlatformInfo, ShellCapabilities};
use std::sync::Mutex;
use tauri::{Manager, State};

type HostState = Mutex<DesktopHost>;

#[tauri::command]
fn platform_info(host: State<'_, HostState>) -> Result<PlatformInfo, String> {
    Ok(host
        .lock()
        .map_err(|_| "desktop host lock poisoned".to_string())?
        .platform()
        .clone())
}

#[tauri::command]
fn content_lock_hash(host: State<'_, HostState>) -> Result<String, String> {
    host.lock()
        .map_err(|_| "desktop host lock poisoned".to_string())?
        .content_lock_hash()
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn content_catalog_page(
    host: State<'_, HostState>,
    offset: Option<u32>,
    limit: Option<u32>,
) -> Result<ActiveCatalogPage, String> {
    host.lock()
        .map_err(|_| "desktop host lock poisoned".to_string())?
        .content_catalog_page(offset, limit)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn content_bundle(host: State<'_, HostState>) -> Result<ContentBundle, String> {
    host.lock()
        .map_err(|_| "desktop host lock poisoned".to_string())?
        .content_bundle()
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn resolve_content_asset(
    host: State<'_, HostState>,
    namespace: String,
    package_id: String,
    asset_id: String,
) -> Result<String, String> {
    host.lock()
        .map_err(|_| "desktop host lock poisoned".to_string())?
        .resolve_content_asset(&namespace, &package_id, &asset_id)
        .map(|path| path.display().to_string())
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn set_match_frozen(host: State<'_, HostState>, frozen: bool) -> Result<(), String> {
    let mut host = host
        .lock()
        .map_err(|_| "desktop host lock poisoned".to_string())?;
    if frozen {
        host.freeze_for_match();
    } else {
        host.unfreeze_after_match();
    }
    Ok(())
}

#[tauri::command]
fn install_package(
    host: State<'_, HostState>,
    package_json: String,
    signature_json: Option<String>,
    verify_key: Option<String>,
    activate: bool,
) -> Result<InstallResult, String> {
    let signature = signature_json
        .as_deref()
        .map(serde_json::from_str::<PackageSignature>)
        .transpose()
        .map_err(|error| format!("invalid package signature: {error}"))?;
    host.lock()
        .map_err(|_| "desktop host lock poisoned".to_string())?
        .install_package(
            package_json.as_bytes(),
            signature.as_ref(),
            verify_key,
            activate,
        )
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn rollback_content(host: State<'_, HostState>) -> Result<(), String> {
    host.lock()
        .map_err(|_| "desktop host lock poisoned".to_string())?
        .rollback()
        .map_err(|error| error.to_string())
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            let host =
                DesktopHost::open_for_shell(data_dir, ShellCapabilities::default(), "tauri2")
                    .map_err(|error| error.to_string())?;
            app.manage(Mutex::new(host));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            platform_info,
            content_lock_hash,
            content_catalog_page,
            content_bundle,
            resolve_content_asset,
            set_match_frozen,
            install_package,
            rollback_content
        ])
        .run(tauri::generate_context!())
        .expect("failed to run noname-next desktop shell");
}
