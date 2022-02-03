//! Textures used in materials

use {
    std::{
        sync::RwLock, 
        collections::HashMap, 
        fmt::{
            self, 
            Formatter
        }, 
        fs::File, 
        io::{
            Cursor, 
            Read
        }, 
        str::SplitWhitespace, 
        sync::Arc
    },
    lazy_static,
    vulkano::{
        sync::GpuFuture, 
        device::Queue, 
        format::Format, 
        image::{
            ImageDimensions, 
            ImmutableImage, 
            MipmapsCount, 
            view::ImageView
        }, 
        memory::pool::{
            PotentialDedicatedAllocation, 
            StdMemoryPoolAlloc
        }, 
        sampler::{
            Filter, 
            MipmapMode, 
            Sampler, 
            SamplerAddressMode
        }
    },
    image::{
        ImageDecoder, 
        bmp::BmpDecoder, 
        jpeg::JpegDecoder, 
        png::PngDecoder
    },
};
lazy_static!{
    static ref DEFAULT_TEXTURES: Arc<RwLock<HashMap<u32, Arc<Texture>>>> = Arc::new(RwLock::new(HashMap::new()));
}

/// A texture is an image that helps describe the surface of a model
pub struct Texture {
    #[allow(clippy::type_complexity)]
    pub img_view: Arc<ImageView<Arc<ImmutableImage<Format, PotentialDedicatedAllocation<StdMemoryPoolAlloc>>>>>,
    pub sampler: Arc<Sampler>,
}

impl fmt::Debug for Texture{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "texture debug todo")
    }
}

impl Texture {
    /// Returns the default Texture <!-- TODO: use instead of undefined texture constant -->
    pub fn default(queue: Arc<Queue>) -> Arc<Self> {
        if let Some(texture) = DEFAULT_TEXTURES.read().unwrap().get(&queue.id_within_family()) { // TODO: other id
            return texture.clone();
        }
        
        let png_bytes = include_bytes!("../../../assets/standard-assets/textures/default_texture.png").to_vec();
        let cursor = Cursor::new(png_bytes);
        let decoder = png::Decoder::new(cursor);
        
        let (info, mut reader) = decoder.read_info().unwrap();
        let dimensions = ImageDimensions::Dim2d {
            width: info.width,
            height: info.height,
            array_layers: 1,
        };

        let mut image_data = Vec::new();
        image_data.resize((info.width * info.height * 4) as usize, 0);
        
        reader.next_frame(&mut image_data).unwrap();
        
        let (image, future) = match ImmutableImage::from_iter(
                    image_data.iter().cloned(),
                    dimensions,
                    MipmapsCount::One,
                    Format::R8G8B8A8Srgb,
                    queue.clone(),
                ) {
            Ok(img) => img,
            _ => unreachable!(),
        };

        Arc::new(future.boxed().then_signal_fence_and_flush().unwrap()).wait(None).unwrap();

        let texture = Arc::new(
                Texture {
                    img_view: ImageView::new(image).unwrap(),
                    sampler: Sampler::new( // TODO
                        queue.device().clone(),
                        Filter::Nearest,
                        Filter::Nearest,
                        MipmapMode::Linear, 
                        SamplerAddressMode::Repeat, // TODO other options
                        SamplerAddressMode::Repeat,
                        SamplerAddressMode::Repeat,
                        0.0,
                        1.0,
                        0.0,
                        0.0
                    ).unwrap()
                }
            );
        
        DEFAULT_TEXTURES.write().unwrap().insert(queue.id_within_family(), texture.clone());
        
        texture
    }

    #[allow(unused_variables, unused_assignments, clippy::unused_io_amount)]
    pub fn from_mtl_line(params: &mut SplitWhitespace, path: &str, queue: Arc<Queue>) -> Result<(Arc<Texture>, Box<dyn GpuFuture>), &'static str> { 
        let mut blendu = Filter::Linear;
        let mut blendv  = Filter::Linear;
        let mut cc = false;
        let mut clamp = SamplerAddressMode::Repeat;
        let mut mm = (0.0, 1.0); // base and gain
        let mut o = (0.0, 0.0, 0.0); // TYPE UVW
        let mut s = (1.0, 1.0, 1.0);
        let mut t = (0.0, 0.0, 0.0);
        let mut texres = None;

        let mut incomplete = String::new();
        for part in params.clone() {
            let part = incomplete + part;
            incomplete = String::new();
            match part.as_str() {
                _ if part.ends_with(".png") => { // color
                    let part = part.replace("\\", "/");
                    let texture_path = path.rsplitn(2, '/').nth(1).unwrap().to_owned() + "/" + part.as_str();
                    let (texture, tex_future) = {
                        let mut file = File::open(texture_path).unwrap();
                        let mut buffer = Vec::new();
                        file.read_to_end(&mut buffer).unwrap();
                        let cursor = Cursor::new(buffer);
                        let decoder = PngDecoder::new(cursor).unwrap();
                        let (width, height) = decoder.dimensions();
                        let mut reader = decoder.into_reader().unwrap();
                        let dimensions = ImageDimensions::Dim2d {
                            width,
                            height,
                            array_layers: 1,
                        };
                        let mut image_data = Vec::new();
                        image_data.resize((width * height * 4) as usize, 0);
                        reader.read_exact(&mut image_data).unwrap();
                
                        let (image, future) = ImmutableImage::from_iter(
                            image_data.iter().cloned(),
                            dimensions,
                            MipmapsCount::One,
                            Format::R8G8B8A8Srgb,
                            queue.clone(),
                        )
                        .unwrap();
            
                        (ImageView::new(image).unwrap(), future)
                    };
                    
                    return Ok((
                        Arc::new(
                            Texture {
                                img_view: texture,
                                sampler: Sampler::new(
                                        queue.device().clone(),
                                        blendu,
                                        blendv,
                                        MipmapMode::Linear, 
                                        clamp,
                                        clamp,
                                        clamp,
                                        0.0,
                                        1.0,
                                        0.0,
                                        0.0
                                    ).unwrap(),
                                // multiplier
                            }
                        ), 
                        tex_future.boxed()
                    ));
                },

                // compiled procedural texture files
                _ if part.ends_with(".jpg") || part.ends_with(".jpeg") => {
                    let part = part.replace("\\", "/");
                    let texture_path = path.rsplitn(2, '/').nth(1).unwrap().to_owned() + "/" + part.as_str();
                    let (texture, tex_future) = {
                        let mut file = File::open(&texture_path).unwrap_or_else(|_| panic!("The texture \"{}\" does not exist.", texture_path));
                        let mut buffer = Vec::new();
                        file.read_to_end(&mut buffer).unwrap();
                        let cursor = Cursor::new(buffer);
                        let decoder = JpegDecoder::new(cursor).unwrap();
                        let (width, height) = decoder.dimensions();
                        let mut reader = decoder.into_reader().unwrap();
                        let dimensions = ImageDimensions::Dim2d {
                            width,
                            height,
                            array_layers: 1,
                        };
                        let mut image_data = Vec::new();
                        image_data.resize((width * height * 4) as usize, 0);
                        reader.read(&mut image_data).unwrap();
                
                        let (image, future) = ImmutableImage::from_iter(
                            image_data.iter().cloned(),
                            dimensions,
                            MipmapsCount::One,
                            Format::R8G8B8Srgb,
                            queue.clone(),
                        )
                        .unwrap();
            
                        (ImageView::new(image).unwrap(), future)
                    };
                    return Ok((
                        Arc::new(
                            Texture {
                                img_view: texture,
                                sampler: Sampler::new(
                                        queue.device().clone(),
                                        blendu,
                                        blendv,
                                        MipmapMode::Linear, 
                                        clamp,
                                        clamp,
                                        clamp,
                                        0.0,
                                        1.0,
                                        0.0,
                                        0.0
                                    ).unwrap(),
                                // multiplier
                            }
                        ), 
                        tex_future.boxed()
                    ));
                },
                
                // compiled procedural texture files
                _ if part.ends_with(".bmp") => { // color
                    let part = part.replace("\\", "/");
                    let texture_path = path.rsplitn(2, '/').nth(1).unwrap().to_owned() + "/" + part.as_str();
                    let (texture, tex_future) = {
                        let mut file = File::open(texture_path).unwrap();
                        let mut buffer = Vec::new();
                        file.read_to_end(&mut buffer).unwrap();
                        let cursor = Cursor::new(buffer);
                        let decoder = BmpDecoder::new(cursor).unwrap();
                        let (width, height) = decoder.dimensions();
                        let mut reader = decoder.into_reader().unwrap();
                        let dimensions = ImageDimensions::Dim2d {
                            width,
                            height,
                            array_layers: 1, // * scale (s)
                        };
                        let mut image_data = Vec::new();
                        image_data.resize((width * height * 4) as usize, 0);
                        reader.read_exact(&mut image_data).unwrap();
                
                        let (image, future) = ImmutableImage::from_iter(
                            image_data.iter().cloned(),
                            dimensions,
                            MipmapsCount::One,
                            Format::R8G8B8Srgb,
                            queue.clone(),
                        ).unwrap();
            
                        (ImageView::new(image).unwrap(), future)
                    };
                    return Ok((
                        Arc::new(
                            Texture {
                                img_view: texture,
                                sampler: Sampler::new(
                                        queue.device().clone(),
                                        blendu,
                                        blendv,
                                        MipmapMode::Linear, 
                                        clamp,
                                        clamp,
                                        clamp,
                                        0.0,
                                        1.0,
                                        0.0,
                                        0.0
                                    ).unwrap(),
                                // multiplier
                            }
                        ), 
                        tex_future.boxed()
                    ));
                },

                "-blendu" => blendu = match params.next().unwrap() {
                    // turns texture blending in the horizontal direction
                    "on" => Filter::Nearest,
                    "off" => Filter::Linear,
                    _ => panic!("formatting error in {}", path)
                },
                "-blendv" => blendv = match params.next().unwrap() {
                    // turns texture blending in the vertical direction
                    "on" => Filter::Nearest,
                    "off" => Filter::Linear,
                    _ => panic!("formatting error in {}", path)
                },
                "-cc" => cc = match params.next().unwrap() {
                    // color correction for the texture
                    "on" => true,
                    "off" => false,
                    _ => panic!("formatting error in {}", path)
                },
                "-clamp" => clamp = match params.next().unwrap() {
                    // clamping on means that only one copy of the texture is mapped onto the surface
                    // rather than repeating copies

                    // When clamping is on, textures are restricted to 0-1 in the uvw range
                    "on" => SamplerAddressMode::ClampToEdge,
                    "off" => SamplerAddressMode::Repeat,
                    _ => panic!("formatting error in {}", path)
                },
                //-mm option modifies the range over which scalar or color texture values may vary.
                // base -> adds a base value to the texture values + increase - decrease/dim 
                // gain -> increases range of texture values
                "-mm" => mm = (
                    params.next().unwrap().parse::<f32>().unwrap_or_else(|_| panic!("formatting error in {}", path)),
                    params.next().unwrap().parse::<f32>().unwrap_or_else(|_| panic!("formatting error in {}", path))
                ),
                
                // horizontal
                // vertical
                // depth 
                // for \/\/\/
                "-o" => o = ( // offset position of texture map
                    params.next().unwrap().parse::<f32>().unwrap_or_else(|_| panic!("formatting error in {}", path)),
                    params.next().unwrap().parse::<f32>().unwrap_or_else(|_| panic!("formatting error in {}", path)), // TODO: optional
                    params.next().unwrap().parse::<f32>().unwrap_or_else(|_| panic!("formatting error in {}", path)), // --
                ),
                "-s" => s = ( // scales the texture pattern
                    params.next().unwrap().parse::<f32>().unwrap_or_else(|_| panic!("formatting error in {}", path)),
                    params.next().unwrap().parse::<f32>().unwrap_or_else(|_| panic!("formatting error in {}", path)), // --
                    params.next().unwrap().parse::<f32>().unwrap_or_else(|_| panic!("formatting error in {}", path)), // --
                ),
                "-t" => t = ( // turbulence for textures -> no noticeable tiling
                    params.next().unwrap().parse::<f32>().unwrap_or_else(|_| panic!("formatting error in {}", path)),
                    params.next().unwrap().parse::<f32>().unwrap_or_else(|_| panic!("formatting error in {}", path)), // --
                    params.next().unwrap().parse::<f32>().unwrap_or_else(|_| panic!("formatting error in {}", path)), // --
                ),

                // the resolution of the texture
                "-texres" => texres = Some(params.next().unwrap().parse::<u32>().unwrap_or_else(|_| panic!("formatting error in {}", path))),

                _ => incomplete = format!("{} ", part),
            }
        }
        Err("No image data found") // TODO clean errors
    }
}