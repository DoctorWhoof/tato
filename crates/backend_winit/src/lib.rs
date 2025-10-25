//! Winit + WGPU backend implementation for the Tato game engine

use std::sync::Arc;
use std::time::Instant;

use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;
use tato::prelude::*;
use tato::{
    arena::*,
    avgbuffer::AvgBuffer,
    backend::{Backend, TextureId},
    Tato,
};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
}

impl Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

const QUAD_VERTICES: &[Vertex] = &[
    Vertex {
        position: [-1.0, -1.0, 0.0],
        tex_coords: [0.0, 1.0],
    },
    Vertex {
        position: [1.0, -1.0, 0.0],
        tex_coords: [1.0, 1.0],
    },
    Vertex {
        position: [1.0, 1.0, 0.0],
        tex_coords: [1.0, 0.0],
    },
    Vertex {
        position: [-1.0, 1.0, 0.0],
        tex_coords: [0.0, 0.0],
    },
];

const QUAD_INDICES: &[u16] = &[0, 1, 2, 2, 3, 0];

pub struct WinitBackend {
    // Window and graphics
    pub event_loop: Option<EventLoop<()>>,
    pub window: Option<Arc<Window>>,
    pub surface: Option<wgpu::Surface<'static>>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,

    // Rendering pipeline
    pub render_pipeline: wgpu::RenderPipeline,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub uniform_buffer: wgpu::Buffer,
    pub bind_group_layout: wgpu::BindGroupLayout,

    // Textures
    pub textures: Vec<wgpu::Texture>,
    pub texture_views: Vec<wgpu::TextureView>,
    pub texture_bind_groups: Vec<wgpu::BindGroup>,
    pub sampler: wgpu::Sampler,

    // State
    pub bg_color: RGBA32,
    pub canvas_rect: Option<Rect<i16>>,
    pub should_close: bool,
    pub integer_scaling: bool,

    // Input
    pub mouse_pos: Vec2<i16>,
    pub pressed_key: Option<Key>,
    pub pad_state: tato::pad::AnaloguePad,

    // Draw operations
    pub draw_ops: Buffer<TempID<DrawOp, u32>, u32>,
    pub draw_ops_additional: Buffer<TempID<DrawOp, u32>, u32>,

    // Performance tracking
    pub pixel_iter_elapsed_time: f32,
    pub drawing_elapsed_time: f32,
    pub buffer_iter_time: AvgBuffer<120, f64>,
    pub buffer_canvas_time: AvgBuffer<120, f64>,

    // Canvas
    pub canvas_texture_id: TextureId,
    pub pixels: Vec<u8>,
}

impl WinitBackend {
    pub async fn new<const LEN: usize>(tato: &Tato, frame_arena: &mut Arena<LEN>) -> Self {
        let event_loop = EventLoop::new().expect("Failed to create event loop");

        // Create window
        let window_attributes = Window::default_attributes()
            .with_title("Tato Demo")
            .with_inner_size(LogicalSize::new(
                tato.video.width() as f64 * 3.0,
                tato.video.height() as f64 * 3.0,
            ));

        // Note: In real usage, window creation should happen in the resumed() callback
        // This is a simplified version for demonstration
        #[allow(deprecated)]
        let window = Arc::new(event_loop.create_window(window_attributes).expect("Failed to create window"));

        // Initialize WGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone()).expect("Failed to create surface");

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("Failed to find suitable adapter");

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: wgpu::MemoryHints::default(),
                    label: None,
                },
                None,
            )
            .await
            .expect("Failed to create device");

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let size = window.inner_size();
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        // Create shader
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        // Create bind group layout
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("texture_bind_group_layout"),
        });

        // Create render pipeline
        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        // Create vertex buffer
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(QUAD_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        // Create index buffer
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(QUAD_INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        // Create uniform buffer (for future use)
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Uniform Buffer"),
            size: 64, // Space for 4x4 matrix
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create sampler
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let pixels = vec![0u8; tato.video.width() as usize * tato.video.height() as usize * 4];

        let mut backend = Self {
            event_loop: Some(event_loop),
            window: Some(window),
            surface: Some(surface),
            device,
            queue,
            config,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            uniform_buffer,
            bind_group_layout,
            textures: Vec::new(),
            texture_views: Vec::new(),
            texture_bind_groups: Vec::new(),
            sampler,
            bg_color: RGBA32 { r: 16, g: 16, b: 16, a: 255 },
            canvas_rect: None,
            should_close: false,
            integer_scaling: true,
            mouse_pos: Vec2::new(0, 0),
            pressed_key: None,
            pad_state: tato::pad::AnaloguePad::default(),
            draw_ops: Buffer::new(frame_arena, 1000).unwrap(),
            draw_ops_additional: Buffer::default(),
            pixel_iter_elapsed_time: 0.0,
            drawing_elapsed_time: 0.0,
            buffer_iter_time: AvgBuffer::new(),
            buffer_canvas_time: AvgBuffer::new(),
            canvas_texture_id: 0,
            pixels,
        };

        // Create canvas texture
        let canvas_texture_id = backend.create_texture(tato.video.width() as i16, tato.video.height() as i16);
        backend.canvas_texture_id = canvas_texture_id;

        backend
    }

    pub fn run<F>(mut self, update_fn: F)
    where
        F: FnMut(&mut Self) + 'static,
    {
        let event_loop = self.event_loop.take().expect("Event loop already taken");

        struct App<F> {
            backend: Option<WinitBackend>,
            update_fn: F,
        }

        impl<F> ApplicationHandler for App<F>
        where
            F: FnMut(&mut WinitBackend),
        {
            fn resumed(&mut self, _event_loop: &ActiveEventLoop) {
                // Application is ready
            }

            fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
                if let Some(backend) = &mut self.backend {
                    match event {
                        WindowEvent::CloseRequested => {
                            backend.should_close = true;
                            event_loop.exit();
                        }
                        WindowEvent::Resized(physical_size) => {
                            backend.resize(physical_size.width, physical_size.height);
                        }
                        WindowEvent::RedrawRequested => {
                            (self.update_fn)(backend);

                            if backend.should_close {
                                event_loop.exit();
                            }
                        }
                        WindowEvent::KeyboardInput {
                            event: KeyEvent { physical_key, state, .. },
                            ..
                        } => {
                            if state == ElementState::Pressed {
                                backend.pressed_key = match physical_key {
                                    PhysicalKey::Code(KeyCode::Tab) => Some(Key::Tab),
                                    PhysicalKey::Code(KeyCode::Backquote) => Some(Key::Grave),
                                    PhysicalKey::Code(KeyCode::Equal) => Some(Key::Plus),
                                    PhysicalKey::Code(KeyCode::Minus) => Some(Key::Minus),
                                    PhysicalKey::Code(KeyCode::Enter) => Some(Key::Enter),
                                    PhysicalKey::Code(KeyCode::Backspace) => Some(Key::Backspace),
                                    PhysicalKey::Code(KeyCode::Delete) => Some(Key::Delete),
                                    PhysicalKey::Code(KeyCode::ArrowLeft) => Some(Key::Left),
                                    PhysicalKey::Code(KeyCode::ArrowRight) => Some(Key::Right),
                                    PhysicalKey::Code(KeyCode::ArrowUp) => Some(Key::Up),
                                    PhysicalKey::Code(KeyCode::ArrowDown) => Some(Key::Down),
                                    _ => None,
                                };
                            }
                        }
                        WindowEvent::CursorMoved { position, .. } => {
                            backend.mouse_pos = Vec2::new(position.x as i16, position.y as i16);
                        }
                        _ => {}
                    }

                    if let Some(window) = &backend.window {
                        window.request_redraw();
                    }
                }
            }
        }

        let mut app = App {
            backend: Some(self),
            update_fn,
        };

        event_loop.run_app(&mut app).expect("Event loop failed");
    }

    fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            if let Some(surface) = &self.surface {
                surface.configure(&self.device, &self.config);
            }
        }
    }

    #[allow(dead_code)]
    fn winit_key_to_tato_key(keycode: KeyCode) -> Option<Key> {
        match keycode {
            KeyCode::Tab => Some(Key::Tab),
            KeyCode::Backquote => Some(Key::Grave),
            KeyCode::Equal => Some(Key::Plus),
            KeyCode::Minus => Some(Key::Minus),
            KeyCode::Enter => Some(Key::Enter),
            KeyCode::Backspace => Some(Key::Backspace),
            KeyCode::Delete => Some(Key::Delete),
            KeyCode::ArrowLeft => Some(Key::Left),
            KeyCode::ArrowRight => Some(Key::Right),
            KeyCode::ArrowUp => Some(Key::Up),
            KeyCode::ArrowDown => Some(Key::Down),
            _ => None,
        }
    }
}

impl Backend for WinitBackend {
    fn clear(&mut self, color: RGBA32) {
        self.bg_color = color;
    }

    fn frame_start<const LEN: usize>(&mut self, frame_arena: &mut Arena<LEN>) {
        self.draw_ops = Buffer::new(frame_arena, 1000).unwrap();
        self.pressed_key = None;
    }

    fn frame_present<'a, const LEN: usize, T>(
        &mut self,
        _frame_arena: &'a mut Arena<LEN>,
        tato: &'a Tato,
        bg_banks: &[&'a T],
    ) where
        &'a T: Into<TilemapRef<'a>>,
    {
        let time_profile = Instant::now();

        // Copy pixels from video chip
        for (i, color) in tato.iter_pixels(bg_banks).enumerate() {
            let index = i * 4;
            self.pixels[index] = color.r;
            self.pixels[index + 1] = color.g;
            self.pixels[index + 2] = color.b;
            self.pixels[index + 3] = color.a;
        }
        self.buffer_iter_time.push(time_profile.elapsed().as_secs_f64());

        // Update canvas texture
        let canvas_texture_id = self.canvas_texture_id;
        let pixels = self.pixels.clone();
        self.update_texture(canvas_texture_id, &pixels);

        // Render frame
        if let Some(surface) = &self.surface {
            let output = match surface.get_current_texture() {
                Ok(output) => output,
                Err(wgpu::SurfaceError::Lost) => {
                    // Reconfigure surface if lost
                    surface.configure(&self.device, &self.config);
                    return;
                }
                Err(wgpu::SurfaceError::OutOfMemory) => {
                    panic!("Out of memory!");
                }
                Err(e) => {
                    eprintln!("Surface error: {:?}", e);
                    return;
                }
            };

            let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

            let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

            {
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: self.bg_color.r as f64 / 255.0,
                                g: self.bg_color.g as f64 / 255.0,
                                b: self.bg_color.b as f64 / 255.0,
                                a: self.bg_color.a as f64 / 255.0,
                            }),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    occlusion_query_set: None,
                    timestamp_writes: None,
                });

                render_pass.set_pipeline(&self.render_pipeline);
                render_pass.set_bind_group(0, &self.texture_bind_groups[self.canvas_texture_id], &[]);
                render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(0..QUAD_INDICES.len() as u32, 0, 0..1);
            }

            self.queue.submit(std::iter::once(encoder.finish()));
            output.present();
        }

        self.buffer_canvas_time.push(time_profile.elapsed().as_secs_f64());
    }

    fn should_close(&self) -> bool {
        self.should_close
    }

    fn set_additional_draw_ops(&mut self, draw_ops: Buffer<TempID<DrawOp, u32>, u32>) {
        self.draw_ops_additional = draw_ops;
    }

    fn measure_text(&self, text: &str, font_size: f32) -> (f32, f32) {
        // Simple approximation - in a real implementation you'd use a font library
        let char_width = font_size * 0.6;
        let char_height = font_size;
        (text.len() as f32 * char_width, char_height)
    }

    fn create_texture(&mut self, width: i16, height: i16) -> TextureId {
        let size = wgpu::Extent3d {
            width: width as u32,
            height: height as u32,
            depth_or_array_layers: 1,
        };

        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
            label: Some("texture_bind_group"),
        });

        let id = self.textures.len();
        self.textures.push(texture);
        self.texture_views.push(view);
        self.texture_bind_groups.push(bind_group);
        id
    }

    fn update_texture(&mut self, id: TextureId, pixels: &[u8]) {
        if id < self.textures.len() {
            let texture = &self.textures[id];
            let size = texture.size();

            self.queue.write_texture(
                wgpu::ImageCopyTexture {
                    texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                pixels,
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * size.width),
                    rows_per_image: Some(size.height),
                },
                size,
            );
        }
    }

    fn get_mouse(&self) -> Vec2<i16> {
        self.mouse_pos
    }

    fn get_pressed_key(&self) -> Option<Key> {
        self.pressed_key
    }

    // fn update_input(&mut self, pad: &mut tato::pad::AnaloguePad) {
    //     // Update pad state - for now just copy, but could implement gamepad support
    //     *pad = self.pad_state;
    // }

    fn set_window_title(&mut self, title: &str) {
        if let Some(window) = &self.window {
            window.set_title(title);
        }
    }

    fn set_target_fps(&mut self, _fps: u32) {
        // Winit doesn't have built-in FPS limiting, would need to implement separately
    }

    fn set_bg_color(&mut self, color: RGBA32) {
        self.bg_color = color;
    }

    fn set_canvas_rect(&mut self, canvas_rect: Option<Rect<i16>>) {
        self.canvas_rect = canvas_rect;
    }

    fn get_screen_size(&self) -> Vec2<i16> {
        Vec2::new(self.config.width as i16, self.config.height as i16)
    }

    fn get_pixel_iter_elapsed_time(&self) -> f32 {
        self.pixel_iter_elapsed_time
    }

    fn get_drawing_elapsed_time(&self) -> f32 {
        self.drawing_elapsed_time
    }
}

pub use tato;
pub use winit;
pub use wgpu;
