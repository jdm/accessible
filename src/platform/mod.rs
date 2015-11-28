#[cfg(target_os = "macosx")]
#[path = "osx.rs"]
pub mod implementation;

#[cfg(target_os = "linux")]
#[path = "linux.rs"]
pub mod implementation;
