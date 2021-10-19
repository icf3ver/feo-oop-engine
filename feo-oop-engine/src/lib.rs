#![feature(fn_traits)]
#![feature(core_intrinsics)]


#[macro_use] extern crate lazy_static;

#[macro_use] extern crate global_macro_derive;
#[macro_use] extern crate named_macro_derive;
#[macro_use] extern crate parent_macro_derive;
#[macro_use] extern crate child_macro_derive;
#[macro_use] extern crate drawable_macro_derive;
#[macro_use] extern crate scriptable_macro_derive;
#[macro_use] extern crate gameobject_macro_derive;

pub mod scene;
pub mod components;
pub mod scripting;
pub mod event;
pub mod graphics;
pub mod registration;

pub mod shaders;
pub mod macros;

pub(crate) mod term_ui;

use {
    self::{
        graphics::frame_system::FrameSystem,
        event::UserEvent,
        scripting::globals::EngineGlobals,
        scene::Scene,
        components::texture::Texture,
        registration::id::IDSystem
    },
    std::{
        sync::{
            Arc,
            RwLock,
        },
        any::Any,
    },
    vulkano::{
        device::{
            Device, 
            DeviceExtensions, 
            Queue
        },
        image::{
            view::ImageView,
            ImageUsage
        }, 
        instance::{
            Instance,
        },
        swapchain::{
            self, 
            AcquireError, 
            ColorSpace, 
            FullscreenExclusive, 
            PresentMode, 
            Surface, 
            SurfaceTransform, 
            Swapchain, 
            SwapchainCreationError
        }, 
        sync::{
            self, 
            FlushError, 
            GpuFuture
        }
    },
    vulkano_win::VkSurfaceBuild,
    winit::{
        dpi::PhysicalSize, 
        event::{
            Event, 
            WindowEvent
        }, 
        event_loop::{
            ControlFlow, 
            EventLoop
        },
        platform::run_return::EventLoopExtRunReturn, 
        window::{
            Window, 
            WindowBuilder
        }
    }
};

/// The Engine
pub struct FeoEngine {
    event_loop: EventLoop<UserEvent<Arc<dyn Any + 'static + Send + Sync>>>,
    surface: Arc<Surface<Window>>,
    queue: Arc<Queue>,

    pub scene: Arc<RwLock<Scene>>,

    pub id_system: IDSystem,

    pub globals: EngineGlobals
}

impl FeoEngine {
    /// Initialize a FeoEngine
    /// if you do nat know the device number of your preferred device
    /// input none and you will be prompted to select one. Note the number
    /// associated with this device and pass it in as a parameter on the next
    /// run through
    pub fn init(scene: Arc<RwLock<Scene>>, index: Option<usize>) -> FeoEngine {
        // Vulkano Instance
        let instance = Instance::new(None, &vulkano_win::required_extensions(), None).unwrap();

        // Physical Device
        let physical = term_ui::prompt_physical_device(&instance, index);

        // event loop
        let event_loop = EventLoop::<UserEvent<Arc<dyn Any + Send + Sync>>>::with_user_event();

        // surface
        let surface = {
            let mut builder = WindowBuilder::new();
            builder.window.inner_size = Some(PhysicalSize::new(1024_u32, 512_u32).into());
            builder.build_vk_surface(&event_loop, instance.clone()).unwrap()
        };

        // get access to the device and get graphics queue
        let (_device, queue) = {
            let queue_family = physical
                .queue_families()
                .find(|&q| q.supports_graphics() && surface.is_supported(q).unwrap_or(false))
                .unwrap();

            let device_ext = DeviceExtensions {
                khr_swapchain: true,
                khr_storage_buffer_storage_class: true,
                ..DeviceExtensions::none()
            };

            let features = physical.supported_features();
            // TODO assertions
            
            let (device, mut queues) = Device::new(
                physical,
                features,
                &device_ext,
                [(queue_family, 0.5)].iter().cloned(),
            ).unwrap();

            (device, queues.next().unwrap())
        };

        Texture::default(queue.clone());

        let id_system = IDSystem::default();
        
        FeoEngine {
            globals: EngineGlobals{ // todo fix
                queue: queue.clone(),
                surface: surface.clone(),
                scene: scene.clone(),
                event_loop_proxy: Arc::new(futures::lock::Mutex::new(event_loop.create_proxy())),
                id_system: id_system.clone()
            },

            //instance,
            event_loop,
            surface,
            queue,

            scene,

            id_system,
        }
    }

    pub fn run(&mut self) {
        // get swapchain and images
        let dimensions: [u32; 2] = self.surface.window().inner_size().into();
        let (mut swapchain, _) = {
            let caps = self.surface.capabilities(self.queue.device().physical_device()).unwrap();
            let format = caps.supported_formats[0].0;
            let alpha = caps.supported_composite_alpha.iter().next().unwrap();

            let (swapchain, images) = Swapchain::new(
                self.queue.device().clone(),
                self.surface.clone(),
                caps.min_image_count,
                format,
                dimensions,
                1,
                ImageUsage::color_attachment(),
                &self.queue,
                SurfaceTransform::Identity,
                alpha,
                PresentMode::Fifo,
                FullscreenExclusive::Default,
                true,
                ColorSpace::SrgbNonLinear,
            ).unwrap();

            let images = images
                .into_iter()
                .map(|image| ImageView::new(image).unwrap())
                .collect::<Vec<_>>();
            
            (swapchain, images)
        };

        // Deferred system
        let mut frame_system = FrameSystem::new(self.queue.clone(), swapchain.format(), dimensions);
        
        // Frame Future
        let mut previous_frame_end = Some(sync::now(self.queue.device().clone()).boxed());

        // Event Loop proxy
        let proxy = self.event_loop.create_proxy();
        
        // Self pointer
        let local_self: *mut Self = self;

        self.event_loop.run_return(move | mut event, _, control_flow| {
            // a mutable reference to self
            let local_self = unsafe {&mut *local_self};
    
            // Deal with Event Redundancy
            while let Event::UserEvent(UserEvent::WinitEvent(inner_event)) = event {
                event = match inner_event {
                    Event::UserEvent(boxed_user_event) => Event::UserEvent(*boxed_user_event),
                    Event::NewEvents(start_case) => Event::NewEvents(start_case),
                    Event::WindowEvent { window_id, event } => Event::WindowEvent { window_id, event },
                    Event::DeviceEvent { device_id, event } => Event::DeviceEvent { device_id, event },
                    Event::Suspended => Event::Suspended,
                    Event::Resumed => Event::Resumed,
                    Event::MainEventsCleared => Event::MainEventsCleared,
                    Event::RedrawRequested(window_id) => Event::RedrawRequested(window_id),
                    Event::RedrawEventsCleared => Event::RedrawEventsCleared,
                    Event::LoopDestroyed => Event::LoopDestroyed,
                };
            }

            // Static Event
            let event: Event<'static, UserEvent<Arc<dyn Any + Send + Sync>>> = event.to_static().unwrap();
            
            // Executor for Object event handlers
            let h_executor = {
                let (executor, spawner) = scripting::new_executor_and_spawner(local_self.globals.clone());
                local_self.scene.read().unwrap().spawn_script_handlers(spawner, event.clone());
                executor
            };
            
            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    *control_flow = ControlFlow::Exit;
                },
                Event::UserEvent( UserEvent::RebuildSwapchain ) | 
                Event::WindowEvent {
                    event: WindowEvent::Resized(_),
                    ..
                } => { 
                    // resizing is slower however because no dynamic viewports are used rendering is faster
                    let dimensions: [u32; 2] = local_self.surface.window().inner_size().into();

                    let (new_swapchain, new_images) = 
                        match swapchain.recreate_with_dimensions(dimensions) {
                            Ok(r) => r,
                            Err(SwapchainCreationError::UnsupportedDimensions) => return,
                            Err(e) => panic!("Failed to recreate swapchain: {:?}", e),
                        };

                    let new_images = new_images
                        .into_iter()
                        .map(|image| ImageView::new(image).unwrap())
                        .collect::<Vec<_>>();
                        
                    swapchain = new_swapchain;

                    frame_system.rebuild_dims(&new_images[..]);
                },
                Event::RedrawEventsCleared => {
                    // Clear buffer pool
                    previous_frame_end.as_mut().unwrap().cleanup_finished();

                    // Generate Executor and Spawner for scripts
                    let (executor, spawner) = scripting::new_executor_and_spawner(local_self.globals.clone());
                    local_self.scene.read().unwrap().spawn_script_cores(spawner);

                    // Get the next image
                    let (image_num, suboptimal, acquire_future) =
                        match swapchain::acquire_next_image(swapchain.clone(), None) {
                            Ok(r) => r,
                            Err(AcquireError::OutOfDate) => {
                                proxy.send_event(UserEvent::RebuildSwapchain).unwrap();
                                return;
                            }
                            Err(e) => panic!("Failed to acquire next image: {:?}", e),
                        };
                    
                    // rebuild swapchain if suboptimal
                    if suboptimal { proxy.send_event(UserEvent::RebuildSwapchain).unwrap(); }
                    
                    // Run scripts to completion
                    executor.run(local_self.scene.clone()); // TODO: merge future with other future and start scripts mvd
                    
                    let future = local_self.scene.read().unwrap()
                        .render(local_self.scene.clone(), &mut frame_system, image_num, acquire_future, &mut previous_frame_end)
                        .then_swapchain_present(local_self.queue.clone(), swapchain.clone(), image_num)
                        .then_signal_fence_and_flush();

                    match future {
                        Ok(future) => {
                            previous_frame_end = Some(future.boxed());
                        },
                        Err(FlushError::OutOfDate) => {
                            proxy.send_event(UserEvent::RebuildSwapchain).unwrap();
                            previous_frame_end = Some(sync::now(local_self.queue.device().clone()).boxed());
                        },
                        Err(e) => {
                            println!("Failed to flush future: {:?}", e);
                            previous_frame_end = Some(sync::now(local_self.queue.device().clone()).boxed());
                        }
                    }
                    
                },
                _ => {},
            }
            
            // Force event handlers to Completion
            h_executor.run(local_self.scene.clone());
        });
    }
}
