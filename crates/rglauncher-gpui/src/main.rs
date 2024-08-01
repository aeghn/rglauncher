use app::{RGLApp, RGLAppMsg};
use assets::Assets;
use components::input::*;
use container::sidebar::{SelectDown, SelectUp};
use gpui::*;
use plugindispatcher::{PluginDispatcher, PluginDispatcherMsg};
use state::StateModel;
use theme::Theme;
use tracing::{info, Level};
use window::{blur_window, get_window_options};

pub mod app;
pub mod arguments;
pub mod assets;
pub mod components;
pub mod constants;
pub mod container;
pub mod plugindispatcher;
pub mod state;
pub mod theme;
pub mod window;

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_thread_ids(true)
        .with_timer(tracing_subscriber::fmt::time::time())
        .init();

    let (app_tx, app_rx) = flume::unbounded::<RGLAppMsg>();
    let app_tx1 = app_tx.clone();
    let pd = PluginDispatcher::new(app_tx1);
    let pd_tx = pd.dp_tx.clone();

    App::new().with_assets(Assets).run(|cx: &mut AppContext| {
        cx.bind_keys([
            KeyBinding::new("backspace", Backspace, None),
            KeyBinding::new("delete", Delete, None),
            KeyBinding::new("left", Left, None),
            KeyBinding::new("right", Right, None),
            KeyBinding::new("shift-left", SelectLeft, None),
            KeyBinding::new("shift-right", SelectRight, None),
            KeyBinding::new("cmd-a", SelectAll, None),
            KeyBinding::new("home", Home, None),
            KeyBinding::new("end", End, None),
            KeyBinding::new("ctrl-cmd-space", ShowCharacterPalette, None),
            KeyBinding::new("cmd-q", Quit, None),
            KeyBinding::new("up", SelectUp, None),
            KeyBinding::new("down", SelectDown, None),

        ]);

        // Bring the menu bar to the foreground (so you can see the menu bar)
        cx.activate(true);
        // Register the `quit` function so it can be referenced by the `MenuItem::action` in the menu bar
        cx.on_action(quit);
        // Add menu items
        cx.set_menus(vec![Menu {
            name: "set_menus".into(),
            items: vec![MenuItem::action("Quit", Quit)],
        }]);

        let options = get_window_options(cx);
        cx.open_window(options, |cx| {
            blur_window(cx);
            StateModel::init(cx);
            Theme::init(cx);

            {
                let app_rx = app_rx.clone();

                cx.spawn(|mut acx| async move {
                    while let Ok(r) = app_rx.recv_async().await {
                        match r {
                            RGLAppMsg::PluginItems(new) => {
                                StateModel::update_async(
                                    |state, cx| state.swap_items(new, cx),
                                    &mut acx,
                                );
                            }
                            _ => {}
                        }
                    }
                })
                .detach();
            }

            RGLApp::new(cx, pd_tx, app_rx)
        })
        .unwrap();
    });
}

// Associate actions using the `actions!` macro (or `impl_actions!` macro)
actions!(set_menus, [Quit]);

// Define the quit function that is registered with the AppContext
fn quit(_: &Quit, cx: &mut AppContext) {
    println!("Gracefully quitting the application . . .");
    cx.quit();
}
