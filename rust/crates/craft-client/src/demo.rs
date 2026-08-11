//! Headless offscreen demo recorder.
//!
//! Renders the *networked* Craft world to a sequence of PNG frames with no
//! window or display, so CI / islo sandboxes can produce a real video of the
//! all-Rust game. It drives the full stack end to end:
//!
//!   * connects a live [`OnlineWorld`] to `craft-server` (protocol + db),
//!   * spawns a second networked "bot" client that walks and builds a tower,
//!   * meshes the authoritative world with the same core mesher + AO the
//!     interactive renderer uses,
//!   * renders block geometry, remote-player cubes and the HUD crosshair with
//!     the same WGSL shaders as `main.rs`,
//!   * reads the frame back off the GPU and writes `frame_00000.png` ...
//!
//! An orbit camera and a day/night cycle animate across the clip.

use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use bytemuck::{Pod, Zeroable};
use craft_client::online::OnlineWorld;
use craft_core::cube::make_cube;
use craft_core::item::{ITEMS, STONE};
use craft_core::matrix::set_matrix_3d;
use craft_core::mesh::mesh_map;
use log::info;
use wgpu::util::DeviceExt;

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
    // WGSL rounds the struct (trailing vec3 `_pad`) up to a 112-byte stride;
    // match it exactly so the bound uniform size satisfies the shader.
    _pad: [f32; 7],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct HudV {
    pos: [f32; 2],
}

fn floats_to_buffer(device: &wgpu::Device, floats: &[f32], label: &str) -> (wgpu::Buffer, u32) {
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
    if verts.is_empty() {
        verts.push(Vertex {
            position: [0.0; 3],
            normal: [0.0; 3],
            uv: [0.0; 2],
            ao: 0.0,
            light: 0.0,
        });
        let buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(label),
            contents: bytemuck::cast_slice(&verts),
            usage: wgpu::BufferUsages::VERTEX,
        });
        return (buf, 0);
    }
    let buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(label),
        contents: bytemuck::cast_slice(&verts),
        usage: wgpu::BufferUsages::VERTEX,
    });
    let n = verts.len() as u32;
    (buf, n)
}

fn peers_buffer(
    device: &wgpu::Device,
    peers: &std::collections::HashMap<i32, craft_client::online::RemotePlayer>,
) -> (wgpu::Buffer, u32) {
    let mut data = Vec::new();
    for p in peers.values() {
        let mut face = vec![0.0f32; 6 * 60];
        let ao = [[0.0f32; 4]; 6];
        let light = [[1.0f32; 4]; 6];
        make_cube(
            &mut face, &ao, &light, 1, 1, 1, 1, 1, 1, p.x, p.y, p.z, 0.4, STONE,
        );
        data.extend_from_slice(&face[..6 * 60]);
    }
    floats_to_buffer(device, &data, "peers")
}

/// Spawn a networked bot that walks in a circle and builds a tower, so the
/// recorded client sees a live peer + authoritative block edits.
fn spawn_bot(addr: String, stop: Arc<AtomicBool>) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        let Ok(mut bot) = OnlineWorld::connect(&addr, "bot") else {
            return;
        };
        let _ = bot.request_chunk(0, 0);
        bot.pump_until(|w| w.blocks_received > 500, Duration::from_secs(10));
        let mut i: u32 = 0;
        let mut tower_y = 14;
        while !stop.load(Ordering::Relaxed) {
            bot.pump();
            let theta = i as f32 * 0.08;
            bot.x = 16.0 + 7.0 * theta.cos();
            bot.z = 16.0 + 7.0 * theta.sin();
            bot.y = 16.0;
            bot.rx = theta;
            let _ = bot.send_position();
            // Build a visible tower near the centre, one block every few ticks.
            if i.is_multiple_of(3) && tower_y < 30 {
                let w = ITEMS[(tower_y as usize) % ITEMS.len()];
                let _ = bot.edit_block(19, tower_y, 19, w);
                tower_y += 1;
            }
            i += 1;
            std::thread::sleep(Duration::from_millis(40));
        }
    })
}

/// Record `frames` PNGs of the live networked world into `out_dir`.
pub fn run(
    texture_path: &Path,
    addr: &str,
    out_dir: &Path,
    frames: u32,
    width: u32,
    height: u32,
) -> Result<(), String> {
    assert!(
        (width * 4).is_multiple_of(256),
        "width*4 must be 256-aligned for readback (got {width})"
    );
    std::fs::create_dir_all(out_dir).map_err(|e| format!("mkdir {}: {e}", out_dir.display()))?;

    // ---- GPU (headless, software-friendly) --------------------------------
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: None,
        force_fallback_adapter: false,
    }))
    .or_else(|| {
        pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::LowPower,
            compatible_surface: None,
            force_fallback_adapter: true,
        }))
    })
    .ok_or_else(|| "no wgpu adapter (need Vulkan/GL; try lavapipe)".to_string())?;
    info!("demo adapter: {:?}", adapter.get_info());
    let (device, queue) = pollster::block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            label: Some("craft-demo"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            memory_hints: Default::default(),
        },
        None,
    ))
    .map_err(|e| format!("device: {e}"))?;

    let format = wgpu::TextureFormat::Rgba8UnormSrgb;

    // ---- atlas texture ----------------------------------------------------
    let img = image::open(texture_path)
        .map_err(|e| format!("texture {}: {e}", texture_path.display()))?
        .into_rgba8();
    let (tw, th) = img.dimensions();
    let atlas = device.create_texture(&wgpu::TextureDescriptor {
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
            texture: &atlas,
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
    let atlas_view = atlas.create_view(&Default::default());
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Nearest,
        min_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    });

    // ---- uniforms + block pipeline ---------------------------------------
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
                resource: wgpu::BindingResource::TextureView(&atlas_view),
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
                    0 => Float32x3, 1 => Float32x3, 2 => Float32x2, 3 => Float32, 4 => Float32
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

    // ---- HUD pipeline -----------------------------------------------------
    let t = 0.004f32;
    let arm = 0.03f32;
    let hud_verts: [HudV; 12] = [
        HudV { pos: [-arm, -t] },
        HudV { pos: [arm, -t] },
        HudV { pos: [arm, t] },
        HudV { pos: [-arm, -t] },
        HudV { pos: [arm, t] },
        HudV { pos: [-arm, t] },
        HudV { pos: [-t, -arm] },
        HudV { pos: [t, -arm] },
        HudV { pos: [t, arm] },
        HudV { pos: [-t, -arm] },
        HudV { pos: [t, arm] },
        HudV { pos: [-t, arm] },
    ];
    let hud_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("hud"),
        contents: bytemuck::cast_slice(&hud_verts),
        usage: wgpu::BufferUsages::VERTEX,
    });
    let hud_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("hud"),
        source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/hud.wgsl").into()),
    });
    let hud_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("hud"),
        bind_group_layouts: &[],
        push_constant_ranges: &[],
    });
    let hud_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("hud"),
        layout: Some(&hud_layout),
        vertex: wgpu::VertexState {
            module: &hud_shader,
            entry_point: Some("vs_main"),
            buffers: &[wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<HudV>() as u64,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &wgpu::vertex_attr_array![0 => Float32x2],
            }],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &hud_shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            ..Default::default()
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
        cache: None,
    });

    // ---- offscreen targets + readback ------------------------------------
    let color = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("color"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let color_view = color.create_view(&Default::default());
    let depth = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("depth"),
        size: wgpu::Extent3d {
            width,
            height,
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
    let bpr = width * 4;
    let readback = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("readback"),
        size: (bpr * height) as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    // ---- live networked world --------------------------------------------
    let mut world = OnlineWorld::connect(addr, "recorder").map_err(|e| format!("connect: {e}"))?;
    world
        .request_chunk(0, 0)
        .map_err(|e| format!("chunk: {e}"))?;
    world.pump_until(
        |w| w.chunks_keyed >= 1 && w.blocks_received > 2000,
        Duration::from_secs(15),
    );
    info!(
        "demo world seeded: blocks={} keyed={}",
        world.blocks_received, world.chunks_keyed
    );

    let stop = Arc::new(AtomicBool::new(false));
    let bot = spawn_bot(addr.to_string(), stop.clone());

    let (mut vbuf, mut vcount) = {
        let (floats, stats) = mesh_map(0, 0, &world.map);
        info!(
            "demo mesh: faces={} ao_sum={:.1}",
            stats.faces, stats.ao_sum
        );
        floats_to_buffer(&device, &floats, "chunk")
    };

    // ---- record frames ----------------------------------------------------
    let center = (16.0f32, 22.0f32, 16.0f32);
    for f in 0..frames {
        // Pump network; remesh + rebuild peers when the world changed.
        world.pump();
        if world.take_dirty() {
            let (floats, _) = mesh_map(0, 0, &world.map);
            let (b, c) = floats_to_buffer(&device, &floats, "chunk");
            vbuf = b;
            vcount = c;
        }
        let (pbuf, pcount) = peers_buffer(&device, &world.players);

        let tt = f as f32 / frames.max(1) as f32;
        let theta = tt * std::f32::consts::TAU * 1.15;
        let radius = 40.0f32;
        let camx = center.0 + radius * theta.cos();
        let camz = center.2 + radius * theta.sin();
        let camy = center.1 + 10.0;
        let (dx, dy, dz) = (center.0 - camx, center.1 - camy, center.2 - camz);
        let len = (dx * dx + dy * dy + dz * dz).sqrt().max(1e-3);
        let ry = (dy / len).asin();
        let rx = dz.atan2(dx) + std::f32::consts::FRAC_PI_2;
        let daylight = 0.55 + 0.45 * (0.5 + 0.5 * (tt * std::f32::consts::TAU).sin());
        let sky = 0.6 + 0.4 * daylight as f64;

        let mut matrix = [0.0f32; 16];
        set_matrix_3d(
            &mut matrix,
            width as i32,
            height as i32,
            camx,
            camy,
            camz,
            rx,
            ry,
            65.0,
            0,
            10,
        );
        let mut m4 = [[0.0f32; 4]; 4];
        for (c, col) in m4.iter_mut().enumerate() {
            for (row, v) in col.iter_mut().enumerate() {
                *v = matrix[c * 4 + row];
            }
        }
        let u = Uniforms {
            matrix: m4,
            camera: [camx, camy, camz],
            fog_distance: (10 * 32 + 64) as f32,
            daylight,
            _pad: [0.0; 7],
        };
        queue.write_buffer(&uniform_buf, 0, bytemuck::bytes_of(&u));

        let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("frame"),
        });
        {
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("block"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &color_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.45 * sky,
                            g: 0.70 * sky,
                            b: 0.95 * sky,
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
            pass.set_pipeline(&pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.set_vertex_buffer(0, vbuf.slice(..));
            pass.draw(0..vcount, 0..1);
            if pcount > 0 {
                pass.set_vertex_buffer(0, pbuf.slice(..));
                pass.draw(0..pcount, 0..1);
            }
        }
        {
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("hud"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &color_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                ..Default::default()
            });
            pass.set_pipeline(&hud_pipeline);
            pass.set_vertex_buffer(0, hud_buf.slice(..));
            pass.draw(0..12, 0..1);
        }
        enc.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &color,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &readback,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(bpr),
                    rows_per_image: Some(height),
                },
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
        queue.submit(Some(enc.finish()));

        let slice = readback.slice(..);
        let (tx, rx_map) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |r| {
            let _ = tx.send(r);
        });
        device.poll(wgpu::Maintain::Wait);
        rx_map
            .recv()
            .map_err(|e| format!("map recv: {e}"))?
            .map_err(|e| format!("map: {e:?}"))?;
        let data = slice.get_mapped_range();
        let path = out_dir.join(format!("frame_{f:05}.png"));
        image::save_buffer(&path, &data, width, height, image::ExtendedColorType::Rgba8)
            .map_err(|e| format!("png {}: {e}", path.display()))?;
        drop(data);
        readback.unmap();

        if f.is_multiple_of(30) {
            info!("demo frame {f}/{frames} peers={pcount} faces={vcount}");
        }
    }

    stop.store(true, Ordering::Relaxed);
    let _ = bot.join();
    info!("demo done: {frames} frames -> {}", out_dir.display());
    Ok(())
}
