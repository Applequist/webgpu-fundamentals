use std::default::Default;
use std::sync::Arc;
use wgpu::{BindGroup, Buffer, BufferAddress, Device, DeviceDescriptor, include_wgsl, InstanceDescriptor, PowerPreference, Queue, RenderPipeline, RequestAdapterOptions, Surface, SurfaceConfiguration};
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::window::Window;

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

        let adapter = instance.request_adapter(&RequestAdapterOptions {
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
            power_preference: PowerPreference::default()
        }).await.unwrap();

        let (device, queue) = adapter.request_device(&DeviceDescriptor {
            label: None,
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
        }, None).await.unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats.iter()
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

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct ColorOffset {
    pub color: [f32; 4],
    pub offset: [f32; 2],
    // this is required to avoid the following error:
    // 'Buffer is bound with size 24 where the shader expects 32 in group[0] compact index 0'
    // See https://sotrh.github.io/learn-wgpu/showcase/alignment/#alignment-of-uniform-and-storage-buffers
    pub padding: [f32; 2],
}

struct Scale {
    pub scale: f32,
}

pub struct View<'a> {
    state: State<'a>,
    object_infos: Vec<Scale>,
    scales_buffer: Buffer,
    bind_group: BindGroup,
    render_pipeline: RenderPipeline,
}

impl<'a> View<'a> {
    pub fn new(window: Arc<Window>) -> Self {
        let state = pollster::block_on(State::new(Arc::clone(&window)));
        let shader = state.device.create_shader_module(include_wgsl!("TriangleShader.wgsl"));
        
        let our_struct_bg_layout = state.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("OurStruct Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }, wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let render_pipeline_layout = state.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&our_struct_bg_layout],
            push_constant_ranges: &[],
        });
        
        let render_pipeline = state.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                compilation_options: Default::default(),
                entry_point: "vs",
                buffers: &[],
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
                unclipped_depth :false,
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

        let num_objects = 100;

        let color_offset_size = std::mem::size_of::<ColorOffset>();
        let color_offset_buffer = state.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(&format!("ColorOffsets buffer")),
            size: (num_objects * color_offset_size) as BufferAddress,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let color_offsets = (0..num_objects).map(|_| {
            ColorOffset {
                color: [rand(0., 1.), rand(0., 1.), rand(0., 1.), 1.0],
                offset: [rand(-0.9, 0.9), rand(-0.9, 0.9)],
                padding: [0.0; 2],
            }
        }).collect::<Vec<_>>();
        state.queue.write_buffer(&color_offset_buffer, 0, bytemuck::cast_slice(&color_offsets));
        
        let scales_size = std::mem::size_of::<[f32; 2]>();
        let scales_buffer = state.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(&format!("Scales buffer")),
            size: (num_objects * scales_size) as BufferAddress,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let object_infos = (0..num_objects).map(|_| {
            let scale = rand(0.2, 0.5);
            Scale {
                scale,
            }
        }).collect::<Vec<_>>();

        let bind_group = state.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!("Triangles Bind Group")),
            layout: &render_pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: color_offset_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: scales_buffer.as_entire_binding()
                }
            ],
        });

        Self {
            state,
            object_infos,
            render_pipeline,
            scales_buffer,
            bind_group,
        }
    }

    pub fn size(&self) -> PhysicalSize<u32> {
        self.state.size()
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
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.state.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
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

            render_pass.set_pipeline(&self.render_pipeline);
            
            let size = self.size();
            let aspect = size.width as f32 / size.height as f32;
            let scales = self.object_infos.iter().map(|obj| {
                let s = obj.scale;
                [s / aspect, s]
            }).collect::<Vec<_>>();
            self.state.queue.write_buffer(&self.scales_buffer, 0, bytemuck::cast_slice(&scales));

            render_pass.set_bind_group(0, &self.bind_group, &[]);
            render_pass.draw(0..3, 0..100);

        }

        self.state.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

fn rand(min: f32, max: f32) -> f32 {
    min + rand::random::<f32>() * (max - min)
}