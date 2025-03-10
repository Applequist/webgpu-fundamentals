use std::default::Default;
use std::f32::consts::PI;
use std::sync::Arc;
use wgpu::{
    include_wgsl, Buffer, BufferAddress, Device, DeviceDescriptor, IndexFormat, InstanceDescriptor,
    PowerPreference, Queue, RenderPipeline, RequestAdapterOptions, Surface, SurfaceConfiguration,
    VertexAttribute, VertexFormat, VertexStepMode,
};
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::window::Window;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    x: f32,
    y: f32,
    color: [u8; 4], // need array of 4 to satisfy bytemuck Pod's requirements.
}

/// Create a circle (or ring) vertices.
pub fn create_circle_vertices(
    inner_radius: f32,
    radius: f32,
    start_angle: f32,
    end_angle: f32,
    num_subdivision: usize,
) -> (Vec<Vertex>, Vec<u32>) {
    let inner_color: [u8; 4] = [255; 4];
    let outer_color: [u8; 4] = [25, 25, 25, 255];

    let vertices: Vec<Vertex> = (0..=num_subdivision)
        .flat_map(|i| {
            let angle: f32 =
                start_angle + (i + 0) as f32 * (end_angle - start_angle) / num_subdivision as f32;

            let (s1, c1) = angle.sin_cos();

            vec![
                Vertex {
                    x: c1 * inner_radius,
                    y: s1 * inner_radius,
                    color: inner_color,
                },
                Vertex {
                    x: c1 * radius,
                    y: s1 * radius,
                    color: outer_color,
                },
            ]
        })
        .collect();
    dbg!(vertices.len());

    let indices: Vec<u32> = (0..num_subdivision as u32)
        .flat_map(|i| {
            let offset = i * 2;
            vec![
                offset,
                offset + 1,
                offset + 2,
                offset + 2,
                offset + 1,
                offset + 3,
            ]
        })
        .collect();
    dbg!(indices.len());
    dbg!(indices.iter().min());
    dbg!(indices.iter().max());
    (vertices, indices)
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct ColorOffset {
    pub color: [u8; 4],
    pub offset: [f32; 2],
}

struct Scale {
    pub scale: f32,
}

pub struct CircleLayer {
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    color_offset_buffer: Buffer,
    object_infos: Vec<Scale>,
    scales_buffer: Buffer,
}

impl CircleLayer {
    pub fn new(state: &State) -> Self {
        let num_objects = 100;

        let color_offset_size = std::mem::size_of::<ColorOffset>();
        let color_offset_buffer = state.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ColorOffsets buffer"),
            size: (num_objects * color_offset_size) as BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let color_offsets = (0..num_objects)
            .map(|_| ColorOffset {
                color: [
                    (rand(0., 1.) * 255.) as u8,
                    (rand(0., 1.) * 255.) as u8,
                    (rand(0., 1.) * 255.) as u8,
                    255,
                ],
                offset: [rand(-0.9, 0.9), rand(-0.9, 0.9)],
            })
            .collect::<Vec<_>>();
        state.queue.write_buffer(
            &color_offset_buffer,
            0,
            bytemuck::cast_slice(&color_offsets),
        );

        let scales_size = std::mem::size_of::<[f32; 2]>();
        let scales_buffer = state.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Scales buffer"),
            size: (num_objects * scales_size) as BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let object_infos = (0..num_objects)
            .map(|_| {
                let scale = rand(0.2, 0.5);
                Scale { scale }
            })
            .collect::<Vec<_>>();

        let (vertices, indices) = create_circle_vertices(0.25, 0.5, 0., 2. * PI, 24);

        let vertex_buffer = state.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Circle Vertex Buffer"),
            size: (vertices.len() * std::mem::size_of::<Vertex>()) as BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        state
            .queue
            .write_buffer(&vertex_buffer, 0, bytemuck::cast_slice(&vertices));

        let index_buffer = state.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Circle Index Buffer"),
            size: (indices.len() * std::mem::size_of::<u32>()) as BufferAddress,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        state
            .queue
            .write_buffer(&index_buffer, 0, bytemuck::cast_slice(&indices));

        Self {
            vertex_buffer,
            index_buffer,
            color_offset_buffer,
            object_infos,
            scales_buffer,
        }
    }
}
pub struct State<'a> {
    surface: Surface<'a>,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    size: PhysicalSize<u32>,
}

impl<'a> State<'a> {
    pub async fn new(window: Arc<Window>) -> State<'a> {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let surface = instance.create_surface(window).unwrap();

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
                power_preference: PowerPreference::default(),
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .filter(|f| f.is_srgb())
            .next()
            .unwrap_or(surface_caps.formats[0]);
        let config = SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            view_formats: vec![],
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            desired_maximum_frame_latency: 2,
        };

        Self {
            // window,
            surface,
            device,
            queue,
            config,
            size,
        }
    }

    pub fn size(&self) -> PhysicalSize<u32> {
        self.size
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }
}

pub struct ViewRenderPass {
    label: String,
    render_pipeline: RenderPipeline,
}

impl ViewRenderPass {
    pub fn new(label: String, state: &State) -> Self {
        let shader = state
            .device
            .create_shader_module(include_wgsl!("TriangleShader.wgsl"));

        let render_pipeline_layout =
            state
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &[],
                    push_constant_ranges: &[],
                });

        let render_pipeline =
            state
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Render Pipeline"),
                    layout: Some(&render_pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        compilation_options: Default::default(),
                        entry_point: "vs",
                        buffers: &[
                            wgpu::VertexBufferLayout {
                                array_stride: 3 * 4,
                                step_mode: VertexStepMode::Vertex,
                                attributes: &[
                                    VertexAttribute {
                                        shader_location: 0, // position
                                        offset: 0,
                                        format: VertexFormat::Float32x2,
                                    },
                                    VertexAttribute {
                                        shader_location: 4, // per_vertex_color
                                        offset: 2 * 4,
                                        format: VertexFormat::Unorm8x4,
                                    },
                                ],
                            },
                            wgpu::VertexBufferLayout {
                                array_stride: 3 * 4,
                                step_mode: VertexStepMode::Instance,
                                attributes: &[
                                    VertexAttribute {
                                        shader_location: 1, // color
                                        offset: 0,
                                        format: VertexFormat::Unorm8x4,
                                    },
                                    VertexAttribute {
                                        shader_location: 2, // offset
                                        offset: 4,
                                        format: VertexFormat::Float32x2,
                                    },
                                ],
                            },
                            wgpu::VertexBufferLayout {
                                array_stride: 2 * 4,
                                step_mode: VertexStepMode::Instance,
                                attributes: &[VertexAttribute {
                                    shader_location: 3, // scales
                                    offset: 0,
                                    format: VertexFormat::Float32x2,
                                }],
                            },
                        ],
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        compilation_options: Default::default(),
                        entry_point: "fs",
                        targets: &[Some(wgpu::ColorTargetState {
                            format: state.config.format,
                            blend: Some(wgpu::BlendState::REPLACE),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
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
                });

        Self {
            label,
            render_pipeline,
        }
    }
}

pub struct View<'a> {
    state: State<'a>,
    passes: Vec<ViewRenderPass>,
    layers: Vec<CircleLayer>,
}

impl<'a> View<'a> {
    pub fn new(window: Arc<Window>) -> Self {
        let state = pollster::block_on(State::new(Arc::clone(&window)));

        let passes = vec![ViewRenderPass::new("Basic View Render Pass".into(), &state)];

        let layers = vec![CircleLayer::new(&state)];
        Self {
            state,
            passes,
            layers,
        }
    }

    pub fn size(&self) -> PhysicalSize<u32> {
        self.state.size()
    }

    pub fn aspect_ratio(&self) -> f32 {
        let size = self.size();
        size.width as f32 / size.height as f32
    }

    /// Reconfigure the [State] whenever the window has been resized.
    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.state.resize(new_size);
    }

    /// Return a bool to indicate whether an event has been fully processed.
    /// If the method returns `true`, the main event loop won't process the event any further.
    pub fn input(&mut self, _event: &WindowEvent) -> bool {
        false
    }

    /// Give a chance to update the view content before rendering.
    /// Don't do anything for now
    pub fn update(&mut self) {
        // TODO
    }

    /// Render the content of the view.
    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.state.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        for pass in &self.passes {
            let mut encoder =
                self.state
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("Render Encoder"),
                    });
            {
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some(&pass.label),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.1,
                                g: 0.2,
                                b: 0.3,
                                a: 1.0,
                            }),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    occlusion_query_set: None,
                    timestamp_writes: None,
                });

                render_pass.set_pipeline(&pass.render_pipeline);

                for layer in &self.layers {
                    let aspect = self.aspect_ratio();
                    let scales = layer
                        .object_infos
                        .iter()
                        .map(|obj| {
                            let s = obj.scale;
                            [s / aspect, s]
                        })
                        .collect::<Vec<_>>();
                    self.state.queue.write_buffer(
                        &layer.scales_buffer,
                        0,
                        bytemuck::cast_slice(&scales),
                    );

                    render_pass.set_vertex_buffer(0, layer.vertex_buffer.slice(..));
                    render_pass.set_vertex_buffer(1, layer.color_offset_buffer.slice(..));
                    render_pass.set_vertex_buffer(2, layer.scales_buffer.slice(..));
                    render_pass.set_index_buffer(layer.index_buffer.slice(..), IndexFormat::Uint32);
                    render_pass.draw_indexed(0..144_u32, 0, 0..100);
                }
            }

            self.state.queue.submit(std::iter::once(encoder.finish()));
        }
        output.present();

        Ok(())
    }
}

fn rand(min: f32, max: f32) -> f32 {
    min + rand::random::<f32>() * (max - min)
}
