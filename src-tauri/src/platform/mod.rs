#[cfg(target_os = "macos")]
pub mod macos;
pub mod ssh_keys;
#[cfg(any(target_os = "linux", test))]
mod unavailable;

pub fn secret_store() -> Box<dyn ghosh_core::secrets::SecretStore> {
    #[cfg(target_os = "macos")]
    {
        Box::new(macos::KeychainStore)
    }
    #[cfg(target_os = "linux")]
    {
        Box::new(unavailable::UnavailableStore)
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        compile_error!("Desktop platform adapter is supported on macOS and Linux only.");
    }
}
