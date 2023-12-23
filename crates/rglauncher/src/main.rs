use crate::invoker::try_communicate;

mod application;
mod arguments;
mod constants;
mod icon_cache;
mod inputbar;
mod invoker;
mod launcher;
mod pluginpreview;
mod preview;
mod resulthandler;
mod sidebar;
mod sidebarrow;
mod window;

fn main() {
    try_communicate().expect("TODO: panic message");
}
