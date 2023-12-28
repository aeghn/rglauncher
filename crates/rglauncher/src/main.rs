use crate::invoker::try_communicate;

mod application;
mod arguments;
mod constants;
mod icon_cache;
mod inputbar;
mod invoker;
mod launcher;
mod pluginpreview;
mod resulthandler;
mod sidebar;
mod sidebarrow;
mod window;

fn main() {
    println!("=============== {:?}", std::env::var("WAYLAND_DISPLAY"));
    try_communicate().expect("TODO: panic message");
}
