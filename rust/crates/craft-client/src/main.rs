//! Walkable local Craft client (wgpu). Wave E finish-line vertical slice.
//!
//! Controls: WASD move, Space jump, mouse look, Esc quit.
//! `--smoke`: mesh one chunk, init GPU if possible, exit (CI-safe when no display).

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use bytemuck::{Pod, Zeroable};
use craft_core::config::{CHUNK_SIZE, WINDOW_HEIGHT, WINDOW_WIDTH};
use craft_core::map::Map;
use craft_core::matrix::set_matrix_3d;
use craft_core::mesh::{fill_chunk_map, mesh_chunk};
use craft_core::physics::{collide_map, get_motion_vector};
use log::{error, info};
use wgpu::util::DeviceExt;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{DeviceEvent, ElementState, KeyEvent, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{CursorGrabMode, Window, WindowId};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
    uv: [f32; 2],
    ao: f32,
    light: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Uniforms {
    matrix: [[f32; 4]; 4],
    camera: [f32; 3],
    fog_distance: f32,
    daylight: f32,
    _pad: [f32; 3],
}

struct RenderState {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    uniform_buf: wgpu::Buffer,
    vertex_buf: wgpu::Buffer,
    vertex_count: u32,
    size: winit::dpi::PhysicalSize<u32>,
}

struct Game {
    window: Option<Arc<Window>>,
    render: Option<RenderState>,
    map: Map,
    x: f32,
    y: f32,
    z: f32,
    rx: f32,
    ry: f32,
    flying: bool,
    keys: Keys,
    mouse_captured: bool,
    last: Instant,
    texture_path: PathBuf,
}

#[derive(Default)]
struct Keys {
    w: bool,
    a: bool,
    s: bool,
    d: bool,
    space: bool,
    shift: bool,
}

impl Game {
    fn new(texture_path: PathBuf) -> Self {
        let map = fill_chunk_map(0, 0);
        // Spawn above terrain at origin.
        let mut y = 40.0f32;
        let mut x = 0.0f32;
        let mut z = 0.0f32;
        for _ in 0..200 {
            y -= 0.2;
            collide_map(&map, 2, &mut x, &mut y, &mut z);
        }
        y += 1.5;
        Self {
            window: None,
            render: None,
            map,
            x,
            y,
            z,
            rx: 0.0,
            ry: 0.0,
            flying: false,
            keys: Keys::default(),
            mouse_captured: false,
            last: Instant::now(),
            texture_path,
        }
    }

    fn rebuild_mesh(device: &wgpu::Device) -> (wgpu::Buffer, u32) {
        let (floats, stats) = mesh_chunk(0, 0);
        info!(
            "mesh chunk(0,0): blocks={} faces={} floats={}",
            stats.blocks, stats.faces, stats.floats
        );
        let mut verts = Vec::with_capacity(floats.len() / 10);
        for chunk in floats.chunks_exact(10) {
            verts.push(Vertex {
                position: [chunk[0], chunk[1], chunk[2]],
                normal: [chunk[3], chunk[4], chunk[5]],
                uv: [chunk[6], chunk[7]],
                ao: chunk[8],
                light: chunk[9],
            });
        }
        let vertex_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("chunk"),
            contents: bytemuck::cast_slice(&verts),
            usage: wgpu::BufferUsages::VERTEX,
        });
        (vertex_buf, verts.len() as u32)
    }

    fn init_gpu(&mut self, window: Arc<Window>) -> Result<(), String> {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        let surface = instance
            .create_surface(window.clone())
            .map_err(|e| format!("surface: {e}"))?;
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .ok_or_else(|| "no wgpu adapter".to_string())?;
        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("craft"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: Default::default(),
            },
            None,
        ))
        .map_err(|e| format!("device: {e}"))?;

        let caps = surface.get_capabilities(&adapter);
        let format = caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let img = image::open(&self.texture_path)
            .map_err(|e| format!("texture {}: {e}", self.texture_path.display()))?
            .into_rgba8();
        let (tw, th) = img.dimensions();
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("atlas"),
            size: wgpu::Extent3d {
                width: tw,
                height: th,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &img,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * tw),
                rows_per_image: Some(th),
            },
            wgpu::Extent3d {
                width: tw,
                height: th,
                depth_or_array_layers: 1,
            },
        );
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let uniform_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("uniforms"),
            size: std::mem::size_of::<Uniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("block"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("block"),
            layout: &bind_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("block"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/block.wgsl").into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("block"),
            bind_group_layouts: &[&bind_layout],
            push_constant_ranges: &[],
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("block"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertex>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![
                        0 => Float32x3,
                        1 => Float32x3,
                        2 => Float32x2,
                        3 => Float32,
                        4 => Float32
                    ],
                }],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let (vertex_buf, vertex_count) = Self::rebuild_mesh(&device);
        self.render = Some(RenderState {
            surface,
            device,
            queue,
            config,
            pipeline,
            bind_group,
            uniform_buf,
            vertex_buf,
            vertex_count,
            size,
        });
        self.window = Some(window);
        Ok(())
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width == 0 || new_size.height == 0 {
            return;
        }
        if let Some(r) = self.render.as_mut() {
            r.size = new_size;
            r.config.width = new_size.width;
            r.config.height = new_size.height;
            r.surface.configure(&r.device, &r.config);
        }
    }

    fn tick(&mut self) {
        let dt = self.last.elapsed().as_secs_f32().min(0.05);
        self.last = Instant::now();
        let mut sz = 0i32;
        let mut sx = 0i32;
        if self.keys.w {
            sz -= 1;
        }
        if self.keys.s {
            sz += 1;
        }
        if self.keys.a {
            sx -= 1;
        }
        if self.keys.d {
            sx += 1;
        }
        let (vx, vy, vz) = get_motion_vector(self.flying, sz, sx, self.rx, self.ry);
        let speed = if self.flying { 20.0 } else { 5.0 } * dt;
        self.x += vx * speed;
        self.z += vz * speed;
        if self.flying {
            self.y += vy * speed;
            if self.keys.space {
                self.y += speed;
            }
            if self.keys.shift {
                self.y -= speed;
            }
        } else {
            // gravity
            self.y -= 9.8 * dt;
            if self.keys.space {
                // tiny jump impulse when grounded — collide will settle
                self.y += 0.35;
            }
        }
        collide_map(&self.map, 2, &mut self.x, &mut self.y, &mut self.z);
        // Stay inside the single loaded chunk for MVP.
        let lo = 0.0;
        let hi = CHUNK_SIZE as f32 - 1.0;
        self.x = self.x.clamp(lo, hi);
        self.z = self.z.clamp(lo, hi);
    }

    fn render_frame(&mut self) {
        let Some(r) = self.render.as_mut() else {
            return;
        };
        let depth = r.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("depth"),
            size: wgpu::Extent3d {
                width: r.config.width,
                height: r.config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let depth_view = depth.create_view(&Default::default());

        let mut matrix = [0.0f32; 16];
        set_matrix_3d(
            &mut matrix,
            r.size.width as i32,
            r.size.height as i32,
            self.x,
            self.y,
            self.z,
            self.rx,
            self.ry,
            65.0,
            0,
            10,
        );
        // Column-major → mat4 as [[f32;4];4] in WGSL is also column-major.
        let mut m4 = [[0.0f32; 4]; 4];
        for c in 0..4 {
            for row in 0..4 {
                m4[c][row] = matrix[c * 4 + row];
            }
        }
        let u = Uniforms {
            matrix: m4,
            camera: [self.x, self.y, self.z],
            fog_distance: (10 * 32 + 64) as f32,
            daylight: 0.85,
            _pad: [0.0; 3],
        };
        r.queue
            .write_buffer(&r.uniform_buf, 0, bytemuck::bytes_of(&u));

        let Ok(frame) = r.surface.get_current_texture() else {
            return;
        };
        let view = frame.texture.create_view(&Default::default());
        let mut encoder = r
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("frame"),
            });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("block"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.45,
                            g: 0.70,
                            b: 0.95,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                ..Default::default()
            });
            pass.set_pipeline(&r.pipeline);
            pass.set_bind_group(0, &r.bind_group, &[]);
            pass.set_vertex_buffer(0, r.vertex_buf.slice(..));
            pass.draw(0..r.vertex_count, 0..1);
        }
        r.queue.submit(Some(encoder.finish()));
        frame.present();
    }
}

impl ApplicationHandler for Game {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let attrs = Window::default_attributes()
            .with_title("Craft (Rust)")
            .with_inner_size(LogicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT));
        let window = Arc::new(event_loop.create_window(attrs).expect("window"));
        match self.init_gpu(window.clone()) {
            Ok(()) => {}
            Err(e) => {
                error!("GPU init failed: {e}");
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => self.resize(size),
            WindowEvent::RedrawRequested => {
                self.tick();
                self.render_frame();
                if let Some(w) = &self.window {
                    w.request_redraw();
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state,
                        ..
                    },
                ..
            } => {
                let down = state == ElementState::Pressed;
                match code {
                    KeyCode::Escape => event_loop.exit(),
                    KeyCode::KeyW => self.keys.w = down,
                    KeyCode::KeyA => self.keys.a = down,
                    KeyCode::KeyS => self.keys.s = down,
                    KeyCode::KeyD => self.keys.d = down,
                    KeyCode::Space => self.keys.space = down,
                    KeyCode::ShiftLeft => self.keys.shift = down,
                    KeyCode::Tab if down => self.flying = !self.flying,
                    _ => {}
                }
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => {
                if let Some(w) = &self.window {
                    let _ = w.set_cursor_grab(CursorGrabMode::Locked);
                    w.set_cursor_visible(false);
                    self.mouse_captured = true;
                }
            }
            _ => {}
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: DeviceEvent,
    ) {
        if !self.mouse_captured {
            return;
        }
        if let DeviceEvent::MouseMotion { delta: (dx, dy) } = event {
            const SENSE: f32 = 0.0025;
            self.rx += dx as f32 * SENSE;
            self.ry -= dy as f32 * SENSE;
            self.ry = self.ry.clamp(-1.5, 1.5);
        }
    }
}

fn find_texture() -> PathBuf {
    let candidates = [
        PathBuf::from("textures/texture.png"),
        PathBuf::from("../textures/texture.png"),
        PathBuf::from("../../textures/texture.png"),
        PathBuf::from("../../../textures/texture.png"),
    ];
    for c in candidates {
        if c.exists() {
            return c;
        }
    }
    PathBuf::from("textures/texture.png")
}

fn main() {
    env_logger::init();
    let smoke = std::env::args().any(|a| a == "--smoke");
    if smoke {
        let (_data, stats) = mesh_chunk(0, 0);
        assert!(stats.faces > 0, "smoke: empty mesh");
        // Best-effort adapter probe (no window — CI / headless safe).
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        match pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::LowPower,
            compatible_surface: None,
            force_fallback_adapter: false,
        })) {
            Some(a) => info!(
                "smoke ok: mesh faces={} adapter={:?}",
                stats.faces,
                a.get_info().name
            ),
            None => info!("smoke ok: mesh faces={} (no GPU adapter)", stats.faces),
        }
        return;
    }
    let event_loop = EventLoop::new().expect("event loop");
    let mut game = Game::new(find_texture());
    event_loop.run_app(&mut game).expect("run");
}
