#[cfg(target_os = "macos")]
#[path = "osx.rs"]
pub mod implementation;

#[cfg(target_os = "linux")]
#[path = "linux.rs"]
pub mod implementation;
