//! Game project.

use fyrox::{
    core::{
        algebra::{Vector2, Vector3},
        color::Color,
        log::Log,
        pool::Handle,
        reflect::prelude::*,
        uuid::Uuid,
        visitor::prelude::*,
    },
    event::{ElementState, Event, WindowEvent},
    graph::SceneGraph,
    gui::{
        border::BorderBuilder,
        brush::Brush,
        button::{Button, ButtonBuilder, ButtonMessage},
        grid::{Column, GridBuilder, Row},
        message::UiMessage,
        text::{Text, TextBuilder, TextMessage},
        widget::{WidgetBuilder, WidgetMessage},
        HorizontalAlignment, Thickness, UiNode, UserInterface, VerticalAlignment,
    },
    material::{Material, MaterialResource},
    plugin::{error::GameResult, Plugin, PluginContext, PluginRegistrationContext},
    resource::texture::Texture,
    scene::{
        base::BaseBuilder,
        camera::{CameraBuilder, OrthographicProjection, Projection},
        sound::{SoundBuffer, SoundBuilder, Status},
        sprite::{Sprite, SpriteBuilder},
        transform::TransformBuilder,
        Scene,
    },
};
use rand::{rngs::ThreadRng, Rng};
use std::collections::HashMap;

// Re-export the engine.
pub use fyrox;

const WINDOW_WIDTH: f32 = 1280.0;
const WINDOW_HEIGHT: f32 = 960.0;
const WORLD_SIZE: f32 = 3200.0;
const HALF_WORLD: f32 = WORLD_SIZE * 0.5;
const OBJECT_SIZE: f32 = 32.0;
const HALF_OBJECT: f32 = OBJECT_SIZE * 0.5;
const DEFAULT_TARGET: usize = 100;
const MAX_TARGET: usize = 5000;
const TARGET_STEP: usize = 100;
const MAX_SPAWNS_PER_FRAME: usize = 25;
const FRAME_TIME: f32 = 0.125;
const FLASH_TIME: f32 = 0.25;
const FPS_REFRESH_TIME: f32 = 0.25;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Mode {
    Menu,
    Benchmark,
}

#[derive(Debug)]
struct MenuUi {
    root: Handle<UiNode>,
    start: Handle<Button>,
    quit: Handle<Button>,
}

#[derive(Debug)]
struct BenchmarkUi {
    root: Handle<UiNode>,
    text: Handle<Text>,
}

#[derive(Debug, Clone)]
struct BenchmarkObject {
    node: Handle<Sprite>,
    position: Vector2<f32>,
    velocity: Vector2<f32>,
    animation_time: f32,
    frame: usize,
    flash_time: f32,
}

#[derive(Default, Visit, Reflect, Debug)]
#[reflect(non_cloneable)]
pub struct Game {
    #[visit(skip)]
    #[reflect(hidden)]
    mode: Option<Mode>,
    #[visit(skip)]
    #[reflect(hidden)]
    scene: Handle<Scene>,
    #[visit(skip)]
    #[reflect(hidden)]
    menu_ui: Option<MenuUi>,
    #[visit(skip)]
    #[reflect(hidden)]
    benchmark_ui: Option<BenchmarkUi>,
    #[visit(skip)]
    #[reflect(hidden)]
    sprite_materials: Vec<MaterialResource>,
    #[visit(skip)]
    #[reflect(hidden)]
    background_material: Option<MaterialResource>,
    #[visit(skip)]
    #[reflect(hidden)]
    objects: Vec<BenchmarkObject>,
    #[visit(skip)]
    #[reflect(hidden)]
    target_count: usize,
    #[visit(skip)]
    #[reflect(hidden)]
    displayed_fps: f32,
    #[visit(skip)]
    #[reflect(hidden)]
    fps_timer: f32,
    #[visit(skip)]
    #[reflect(hidden)]
    overlay_timer: f32,
    #[visit(skip)]
    #[reflect(hidden)]
    rng: Option<ThreadRng>,
}

impl Game {
    fn make_material(path: &str, context: &PluginContext) -> MaterialResource {
        let mut material = Material::standard_sprite();
        material.bind("diffuseTexture", context.resource_manager.request::<Texture>(path));
        MaterialResource::new_ok(Uuid::new_v4(), Default::default(), material)
    }

    fn setup_scene(&mut self, context: &mut PluginContext) {
        let mut scene = Scene::new();

        CameraBuilder::new(
            BaseBuilder::new().with_local_transform(
                TransformBuilder::new()
                    .with_local_position(Vector3::new(0.0, 0.0, 10.0))
                    .build(),
            ),
        )
        .with_projection(Projection::Orthographic(OrthographicProjection {
            z_near: 0.0,
            z_far: 100.0,
            vertical_size: WORLD_SIZE * 0.5,
        }))
        .build(&mut scene.graph);

        self.background_material = Some(Self::make_material(
            "assets/tiles/benchmark_tilemap.png",
            context,
        ));
        self.sprite_materials = (0..8)
            .map(|i| Self::make_material(&format!("assets/sprites/bench_blob_{i}.png"), context))
            .collect();

        if let Some(material) = self.background_material.clone() {
            SpriteBuilder::new(
                BaseBuilder::new()
                    .with_name("Benchmark Tilemap")
                    .with_local_transform(
                        TransformBuilder::new()
                            .with_local_position(Vector3::new(0.0, 0.0, -1.0))
                            .build(),
                    ),
            )
            .with_material(material)
            .with_size(HALF_WORLD)
            .build(&mut scene.graph);
            Log::info("Tilemap created from assets/tiles/benchmark_tilemap.png");
        }

        SoundBuilder::new(BaseBuilder::new().with_name("Benchmark Music"))
            .with_buffer(Some(
                context
                    .resource_manager
                    .request::<SoundBuffer>("assets/audio/benchmark_loop.wav"),
            ))
            .with_looping(true)
            .with_status(Status::Playing)
            .with_spatial_blend_factor(0.0)
            .with_gain(0.45)
            .build(&mut scene.graph);
        Log::info("Background music started from assets/audio/benchmark_loop.wav");

        self.scene = context.scenes.add(scene);
        Log::info("Camera setup complete for 3200x3200 benchmark world");
    }

    fn show_menu(&mut self, context: &mut PluginContext) {
        self.clear_benchmark_ui(context);
        let ui = context.user_interfaces.first_mut();
        let mut ctx = ui.build_ctx();

        let title = TextBuilder::new(
            WidgetBuilder::new()
                .on_row(0)
                .with_height(90.0)
                .with_foreground(Brush::Solid(Color::WHITE).into()),
        )
        .with_text("KAIJU 2D BENCHMARK")
        .with_font_size(52.0.into())
        .with_horizontal_text_alignment(HorizontalAlignment::Center)
        .with_vertical_text_alignment(VerticalAlignment::Center)
        .build(&mut ctx);

        let start = ButtonBuilder::new(
            WidgetBuilder::new()
                .on_row(1)
                .with_width(320.0)
                .with_height(58.0)
                .with_horizontal_alignment(HorizontalAlignment::Center),
        )
        .with_text("Start Benchmark")
        .build(&mut ctx);

        let quit = ButtonBuilder::new(
            WidgetBuilder::new()
                .on_row(2)
                .with_width(320.0)
                .with_height(58.0)
                .with_horizontal_alignment(HorizontalAlignment::Center),
        )
        .with_text("Quit")
        .build(&mut ctx);

        let stack = GridBuilder::new(
            WidgetBuilder::new()
                .with_width(520.0)
                .with_height(260.0)
                .with_horizontal_alignment(HorizontalAlignment::Center)
                .with_vertical_alignment(VerticalAlignment::Center)
                .with_child(title)
                .with_child(start)
                .with_child(quit),
        )
        .add_row(Row::stretch())
        .add_row(Row::strict(68.0))
        .add_row(Row::strict(68.0))
        .add_column(Column::stretch())
        .build(&mut ctx);

        let root = BorderBuilder::new(
            WidgetBuilder::new()
                .with_width(WINDOW_WIDTH)
                .with_height(WINDOW_HEIGHT)
                .with_background(Brush::Solid(Color::opaque(4, 8, 18)).into())
                .with_child(stack),
        )
        .build(&mut ctx);

        self.menu_ui = Some(MenuUi {
            root: root.to_base(),
            start,
            quit,
        });
        self.mode = Some(Mode::Menu);
        Log::info("Main menu shown");
    }

    fn start_benchmark(&mut self, context: &mut PluginContext) {
        self.clear_menu_ui(context);
        self.clear_objects(context);
        self.target_count = DEFAULT_TARGET;
        self.displayed_fps = 0.0;
        self.fps_timer = 0.0;
        self.overlay_timer = 0.0;
        self.mode = Some(Mode::Benchmark);
        self.show_benchmark_ui(context);
        Log::info("Benchmark started");
    }

    fn show_benchmark_ui(&mut self, context: &mut PluginContext) {
        let ui = context.user_interfaces.first_mut();
        let mut ctx = ui.build_ctx();

        let text = TextBuilder::new(
            WidgetBuilder::new()
                .with_margin(Thickness::uniform(14.0))
                .with_foreground(Brush::Solid(Color::WHITE).into()),
        )
        .with_text("FPS: 0\nObjects: 0/100\nA add | Z remove | R reset | Q quit")
        .with_font_size(26.0.into())
        .with_shadow(true)
        .with_shadow_dilation(1.0)
        .build(&mut ctx);

        let root = BorderBuilder::new(
            WidgetBuilder::new()
                .with_width(430.0)
                .with_height(118.0)
                .with_margin(Thickness::uniform(12.0))
                .with_horizontal_alignment(HorizontalAlignment::Left)
                .with_vertical_alignment(VerticalAlignment::Top)
                .with_hit_test_visibility(false)
                .with_background(Brush::Solid(Color::opaque(2, 4, 10)).into())
                .with_child(text),
        )
        .build(&mut ctx);

        self.benchmark_ui = Some(BenchmarkUi {
            root: root.to_base(),
            text,
        });
    }

    fn reset_benchmark(&mut self, context: &mut PluginContext) {
        self.clear_objects(context);
        self.target_count = DEFAULT_TARGET;
        self.displayed_fps = 0.0;
        self.fps_timer = 0.0;
        self.overlay_timer = 0.0;
        Log::info("Benchmark reset");
    }

    fn change_target(&mut self, delta: isize) {
        let next = (self.target_count as isize + delta).clamp(0, MAX_TARGET as isize) as usize;
        if next != self.target_count {
            self.target_count = next;
            Log::info(format!("Target object count changed to {}", self.target_count));
        }
    }

    fn clear_menu_ui(&mut self, context: &mut PluginContext) {
        if let Some(menu) = self.menu_ui.take() {
            context
                .user_interfaces
                .first_mut()
                .send(menu.root, WidgetMessage::Remove);
        }
    }

    fn clear_benchmark_ui(&mut self, context: &mut PluginContext) {
        if let Some(overlay) = self.benchmark_ui.take() {
            context
                .user_interfaces
                .first_mut()
                .send(overlay.root, WidgetMessage::Remove);
        }
    }

    fn clear_objects(&mut self, context: &mut PluginContext) {
        if self.scene.is_some() {
            let scene = &mut context.scenes[self.scene];
            for object in self.objects.drain(..) {
                if scene.graph.is_valid_handle(object.node) {
                    scene.graph.remove_node(object.node);
                }
            }
        } else {
            self.objects.clear();
        }
    }

    fn spawn_object(&mut self, context: &mut PluginContext) {
        let rng = self.rng.get_or_insert_with(rand::thread_rng);
        let x = rng.gen_range((-HALF_WORLD + OBJECT_SIZE)..(HALF_WORLD - OBJECT_SIZE));
        let y = rng.gen_range((-HALF_WORLD + OBJECT_SIZE)..(HALF_WORLD - OBJECT_SIZE));
        let mut vx: f32 = rng.gen_range(-140.0..140.0);
        let mut vy: f32 = rng.gen_range(-140.0..140.0);
        if vx.abs() + vy.abs() < 80.0 {
            vx = 120.0;
            vy = 80.0;
        }

        let frame = rng.gen_range(0..self.sprite_materials.len());
        let node = SpriteBuilder::new(
            BaseBuilder::new()
                .with_name("Bench Blob")
                .with_local_transform(
                    TransformBuilder::new()
                        .with_local_position(Vector3::new(x.round(), y.round(), 0.0))
                        .build(),
                ),
        )
        .with_material(self.sprite_materials[frame].clone())
        .with_size(HALF_OBJECT)
        .build(&mut context.scenes[self.scene].graph);

        self.objects.push(BenchmarkObject {
            node,
            position: Vector2::new(x, y),
            velocity: Vector2::new(vx, vy),
            animation_time: rng.gen_range(0.0..(FRAME_TIME * 8.0)),
            frame,
            flash_time: 0.0,
        });
    }

    fn update_benchmark(&mut self, context: &mut PluginContext) {
        if self.objects.len() > self.target_count {
            let scene = &mut context.scenes[self.scene];
            while self.objects.len() > self.target_count {
                if let Some(object) = self.objects.pop() {
                    scene.graph.remove_node(object.node);
                }
            }
        }

        let spawn_count = (self.target_count - self.objects.len()).min(MAX_SPAWNS_PER_FRAME);
        for _ in 0..spawn_count {
            self.spawn_object(context);
        }
        if spawn_count > 0 && self.target_count >= 1000 && self.objects.len() % 500 < spawn_count {
            Log::info(format!(
                "Object spawn progress: {}/{}",
                self.objects.len(),
                self.target_count
            ));
        }

        let dt = context.dt;
        for object in &mut self.objects {
            object.position += object.velocity * dt;

            let min = -HALF_WORLD + HALF_OBJECT;
            let max = HALF_WORLD - HALF_OBJECT;
            if object.position.x < min {
                object.position.x = min;
                object.velocity.x = object.velocity.x.abs();
            } else if object.position.x > max {
                object.position.x = max;
                object.velocity.x = -object.velocity.x.abs();
            }
            if object.position.y < min {
                object.position.y = min;
                object.velocity.y = object.velocity.y.abs();
            } else if object.position.y > max {
                object.position.y = max;
                object.velocity.y = -object.velocity.y.abs();
            }

            object.animation_time += dt;
            let frame = ((object.animation_time / FRAME_TIME) as usize) % self.sprite_materials.len();
            object.frame = frame;
            object.flash_time = (object.flash_time - dt).max(0.0);
        }

        self.resolve_collisions();
        self.sync_scene_nodes(context);
        self.update_overlay(context);
    }

    fn resolve_collisions(&mut self) {
        let mut grid: HashMap<(i32, i32), Vec<usize>> = HashMap::new();
        for (index, object) in self.objects.iter().enumerate() {
            let cell = Self::cell_for(object.position);
            grid.entry(cell).or_default().push(index);
        }

        for i in 0..self.objects.len() {
            let (cx, cy) = Self::cell_for(self.objects[i].position);
            for oy in -1..=1 {
                for ox in -1..=1 {
                    if let Some(indices) = grid.get(&(cx + ox, cy + oy)) {
                        for &j in indices {
                            if j > i {
                                Self::collide_pair(&mut self.objects, i, j);
                            }
                        }
                    }
                }
            }
        }
    }

    fn cell_for(position: Vector2<f32>) -> (i32, i32) {
        (
            ((position.x + HALF_WORLD) / OBJECT_SIZE).floor() as i32,
            ((position.y + HALF_WORLD) / OBJECT_SIZE).floor() as i32,
        )
    }

    fn collide_pair(objects: &mut [BenchmarkObject], a: usize, b: usize) {
        let (left, right) = objects.split_at_mut(b);
        let first = &mut left[a];
        let second = &mut right[0];
        let dx = second.position.x - first.position.x;
        let dy = second.position.y - first.position.y;
        let overlap_x = OBJECT_SIZE - dx.abs();
        let overlap_y = OBJECT_SIZE - dy.abs();

        if overlap_x > 0.0 && overlap_y > 0.0 {
            if overlap_x < overlap_y {
                let push = overlap_x * 0.5 * dx.signum();
                first.position.x -= push;
                second.position.x += push;
                std::mem::swap(&mut first.velocity.x, &mut second.velocity.x);
            } else {
                let push = overlap_y * 0.5 * dy.signum();
                first.position.y -= push;
                second.position.y += push;
                std::mem::swap(&mut first.velocity.y, &mut second.velocity.y);
            }
            first.flash_time = FLASH_TIME;
            second.flash_time = FLASH_TIME;
        }
    }

    fn sync_scene_nodes(&mut self, context: &mut PluginContext) {
        let scene = &mut context.scenes[self.scene];
        for object in &self.objects {
            if let Ok(sprite) = scene.graph.try_get_mut(object.node) {
                sprite.set_color(if object.flash_time > 0.0 {
                    Color::opaque(255, 255, 255)
                } else {
                    Color::WHITE
                });
                sprite
                    .material_mut()
                    .set_value_and_mark_modified(self.sprite_materials[object.frame].clone());
                sprite
                    .local_transform_mut()
                    .set_position(Vector3::new(object.position.x.round(), object.position.y.round(), 0.0));
            }
        }
    }

    fn update_overlay(&mut self, context: &mut PluginContext) {
        self.fps_timer += context.dt;
        self.overlay_timer += context.dt;
        if context.dt > 0.0 {
            let instant_fps = 1.0 / context.dt;
            self.displayed_fps = if self.displayed_fps <= 0.0 {
                instant_fps
            } else {
                self.displayed_fps * 0.9 + instant_fps * 0.1
            };
        }

        if self.overlay_timer >= FPS_REFRESH_TIME {
            self.overlay_timer = 0.0;
            if let Some(overlay) = &self.benchmark_ui {
                let text = format!(
                    "FPS: {:.0}\nObjects: {}/{}\nA add | Z remove | R reset | Q quit",
                    self.displayed_fps,
                    self.objects.len(),
                    self.target_count
                );
                context
                    .user_interfaces
                    .first_mut()
                    .send(overlay.text, TextMessage::Text(text));
            }
        }
    }
}

impl Plugin for Game {
    fn register(&self, _context: PluginRegistrationContext) -> GameResult {
        Ok(())
    }

    fn init(&mut self, _scene_path: Option<&str>, mut context: PluginContext) -> GameResult {
        Log::info("Game launch");
        self.rng = Some(rand::thread_rng());
        self.setup_scene(&mut context);
        context
            .user_interfaces
            .add(UserInterface::new(Vector2::new(WINDOW_WIDTH, WINDOW_HEIGHT)));
        self.show_menu(&mut context);
        Ok(())
    }

    fn on_deinit(&mut self, mut context: PluginContext) -> GameResult {
        self.clear_objects(&mut context);
        self.clear_menu_ui(&mut context);
        self.clear_benchmark_ui(&mut context);
        Ok(())
    }

    fn update(&mut self, context: &mut PluginContext) -> GameResult {
        if self.mode == Some(Mode::Benchmark) {
            self.update_benchmark(context);
        }
        Ok(())
    }

    fn on_os_event(&mut self, event: &Event<()>, mut context: PluginContext) -> GameResult {
        if let Event::WindowEvent {
            event: WindowEvent::KeyboardInput { event, .. },
            ..
        } = event
        {
            if event.state == ElementState::Pressed {
                match event.physical_key {
                    fyrox::keyboard::PhysicalKey::Code(code) => match (self.mode, code) {
                        (Some(Mode::Menu), fyrox::keyboard::KeyCode::Enter) => {
                            self.start_benchmark(&mut context);
                        }
                        (Some(Mode::Menu), fyrox::keyboard::KeyCode::KeyQ)
                        | (Some(Mode::Benchmark), fyrox::keyboard::KeyCode::KeyQ) => {
                            Log::info("Quit selection");
                            context.loop_controller.exit();
                        }
                        (Some(Mode::Benchmark), fyrox::keyboard::KeyCode::KeyA) => {
                            self.change_target(TARGET_STEP as isize);
                        }
                        (Some(Mode::Benchmark), fyrox::keyboard::KeyCode::KeyZ) => {
                            self.change_target(-(TARGET_STEP as isize));
                        }
                        (Some(Mode::Benchmark), fyrox::keyboard::KeyCode::KeyR) => {
                            self.reset_benchmark(&mut context);
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn on_ui_message(
        &mut self,
        context: &mut PluginContext,
        message: &UiMessage,
        _ui_handle: Handle<UserInterface>,
    ) -> GameResult {
        if let Some(ButtonMessage::Click) = message.data::<ButtonMessage>() {
            if let Some(menu) = &self.menu_ui {
                if message.destination() == menu.start {
                    self.start_benchmark(context);
                } else if message.destination() == menu.quit {
                    Log::info("Quit selection");
                    context.loop_controller.exit();
                }
            }
        }
        Ok(())
    }
}
