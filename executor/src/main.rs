//! Executor with your game connected to it as a plugin.
use fyrox::core::log::Log;
use fyrox::dpi::{PhysicalSize, Size};
use fyrox::engine::{executor::Executor, GraphicsContextParams};
use fyrox::event_loop::EventLoop;
use fyrox::window::WindowAttributes;

fn main() {
    Log::set_file_name("gamebench_fyrox.log");

    let mut window_attributes = WindowAttributes::default();
    window_attributes.title = "Kaiju 2D Benchmark".to_string();
    window_attributes.inner_size = Some(Size::Physical(PhysicalSize::new(1280, 960)));
    window_attributes.min_inner_size = Some(Size::Physical(PhysicalSize::new(1280, 960)));
    window_attributes.max_inner_size = Some(Size::Physical(PhysicalSize::new(1280, 960)));
    window_attributes.resizable = false;

    let mut executor = Executor::from_params(
        Some(EventLoop::new().unwrap()),
        GraphicsContextParams {
            window_attributes,
            vsync: true,
            msaa_sample_count: None,
            graphics_server_constructor: Default::default(),
            named_objects: false,
        },
    );

    // Dynamic linking with hot reloading.
    #[cfg(feature = "dylib")]
    {
        #[cfg(target_os = "windows")]
        let file_name = "game_dylib.dll";
        #[cfg(target_os = "linux")]
        let file_name = "libgame_dylib.so";
        #[cfg(target_os = "macos")]
        let file_name = "libgame_dylib.dylib";
        executor.add_dynamic_plugin(file_name, true, true).unwrap();
    }

    // Static linking.
    #[cfg(not(feature = "dylib"))]
    {
        use gamebench_fyrox::Game;
        executor.add_plugin(Game::default());
    }

    executor.run()
}
