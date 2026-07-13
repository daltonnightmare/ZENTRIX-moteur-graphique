use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop, ControlFlow},
    window::{Window, WindowId},
    dpi::PhysicalSize,
};
use wgpu::util::DeviceExt;

// ------------------------------------------------------------
// 1. RHI (surface avec 'static)
// ------------------------------------------------------------
struct Rhi {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: PhysicalSize<u32>,
}

impl Rhi {
    async fn new(window: &'static Window) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
        let surface = instance.create_surface(window).expect("Surface creation failed");

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await
            .expect("Failed to find an appropriate adapter");

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Device"),
                    ..Default::default()
                },
                None,
            )
            .await
            .expect("Failed to create device");

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        Rhi {
            surface,
            device,
            queue,
            config,
            size,
        }
    }

    fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    fn get_current_texture(&self) -> Result<wgpu::SurfaceTexture, wgpu::SurfaceError> {
        self.surface.get_current_texture()
    }
}

// ------------------------------------------------------------
// 2. MOTEUR (possède la référence statique et la RHI)
// ------------------------------------------------------------
struct Engine {
    window: &'static Window,
    rhi: Rhi,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
}

impl Engine {
    async fn new(window: &'static Window) -> Self {
        let rhi = Rhi::new(window).await;

        let shader = rhi.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Triangle Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let pipeline_layout = rhi
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Pipeline Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        let render_pipeline = rhi
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: rhi.config.format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
                cache: None,
            });

        let vertices: [f32; 9] = [
            -0.5, -0.5, 0.0,
             0.5, -0.5, 0.0,
             0.0,  0.5, 0.0,
        ];
        let vertex_buffer = rhi.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let indices: [u16; 3] = [0, 1, 2];
        let index_buffer = rhi.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        Engine {
            window,
            rhi,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            num_indices: 3,
        }
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let surface_texture = self.rhi.get_current_texture()?;
        let view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .rhi
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
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
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        }

        self.rhi.queue.submit(std::iter::once(encoder.finish()));
        surface_texture.present();
        Ok(())
    }

    fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.rhi.resize(new_size);
    }

    fn request_redraw(&self) {
        self.window.request_redraw();
    }
}

// ------------------------------------------------------------
// 3. APPLICATION (gère le cycle de vie)
// ------------------------------------------------------------
struct App {
    engine: Option<Engine>,
}

impl Default for App {
    fn default() -> Self {
        Self { engine: None }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes()
            .with_title("Test wgpu API")
            .with_inner_size(PhysicalSize::new(800, 600));
        let window = event_loop
            .create_window(window_attributes)
            .expect("Failed to create window");

        // Leak la fenêtre pour obtenir une référence 'static
        let window_box = Box::new(window);
        let window_ref: &'static Window = Box::leak(window_box);

        let engine = pollster::block_on(Engine::new(window_ref));
        self.engine = Some(engine);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                println!("Fermeture demandée");
                event_loop.exit();
            }
            WindowEvent::Resized(new_size) => {
                if let Some(engine) = self.engine.as_mut() {
                    engine.resize(new_size);
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(engine) = self.engine.as_mut() {
                    if let Err(e) = engine.render() {
                        match e {
                            wgpu::SurfaceError::Lost => {
                                let size = engine.window.inner_size();
                                engine.resize(size);
                            }
                            wgpu::SurfaceError::OutOfMemory => {
                                eprintln!("Out of memory, exiting");
                                event_loop.exit();
                            }
                            _ => eprintln!("Render error: {:?}", e),
                        }
                    }
                }
                if let Some(engine) = self.engine.as_ref() {
                    engine.request_redraw();
                }
            }
            _ => {}
        }
    }
}

// ------------------------------------------------------------
// 4. POINT D'ENTRÉE
// ------------------------------------------------------------
fn main() {
    env_logger::init();

    let event_loop = EventLoop::new().expect("Event loop creation failed");
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::default();
    event_loop.run_app(&mut app).expect("Event loop run failed");
}