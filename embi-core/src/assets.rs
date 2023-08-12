use std::sync::mpsc::Sender;

use crate::*;

type BasePathFn = fn(&str) -> String;

pub static ASSETS: Lazy<AtomicRefCell<Assets>> =
    Lazy::new(|| AtomicRefCell::new(Assets::new()));

pub fn texture_id_safe(id: &str) -> Option<TextureHandle> {
    ASSETS.borrow().textures.get(id).copied()
}

pub struct LoadSoundRequest {
    pub path: String,
    pub handle: Sound,
    pub bytes: Vec<u8>,
}

pub struct LoadRequest {
    pub path: String,
    pub handle: TextureHandle,
    pub bytes: Vec<u8>,
}


pub struct LoadedImage {
    pub path: String,
    pub handle: TextureHandle,
    pub image: image::DynamicImage,
}

pub struct AssetSource {
    pub dir: &'static include_dir::Dir<'static>,
    pub base_path: BasePathFn,
}

pub fn texture_id_unchecked(id: &str) -> TextureHandle {
    TextureHandle::key_unchecked(id)
}

// TODO: impl Into<Cow<'static, str>>
pub fn texture_id(id: &str) -> TextureHandle {
    if id == "1px" {
        texture_id_safe("1px").expect("1px must be loaded")
    } else {
        texture_id_safe(id).unwrap_or_else(|| {
            if id == "error" {
                for key in ASSETS.borrow().textures.keys().sorted() {
                    println!("{key}");
                }

                panic!("Failed to load error texture with ID = '{}'", id)
            }

            texture_id("error")
        })
    }
}

// TODO: rename to something like "unchecked_id"
pub fn texture_path(path: &str) -> TextureHandle {
    TextureHandle::from_path(path)
}

pub fn sound_id(id: &str) -> Sound {
    ASSETS.borrow().sound_ids.get(id).copied().unwrap_or_else(|| {
        if id == "error" {
            panic!("Failed to load error sound {}", id)
        }

        error!("failed to load sound {}", id);
        sound_id("error")
    })
}

pub fn init_asset_source(
    dir: &'static include_dir::Dir<'static>,
    base_path: fn(&str) -> String,
) {
    ASSETS.borrow_mut().asset_source = Some(AssetSource { dir, base_path });
}

pub struct Assets {
    pub texture_send: Arc<Mutex<std::sync::mpsc::Sender<Vec<LoadRequest>>>>,

    pub textures: HashMap<String, TextureHandle>,
    pub texture_load_queue: Vec<(String, String)>,
    // pub texture_load_bytes_queue: Vec<String>,
    // TODO: private & fix?
    pub texture_image_map:
        Arc<Mutex<HashMap<TextureHandle, image::DynamicImage>>>,

    pub sound_load_queue: Vec<(String, String)>,

    pub sound_ids: HashMap<String, Sound>,
    pub sounds: Arc<Mutex<HashMap<Sound, StaticSoundData>>>,
    pub sound_handles: HashMap<Sound, StaticSoundHandle>,

    pub sound_groups: HashMap<String, Vec<Sound>>,

    pub sound_send: Arc<Mutex<Sender<LoadSoundRequest>>>,

    pub thread_pool: rayon::ThreadPool,
    pub current_queue: Arc<Mutex<Option<TextureLoadQueue>>>,

    pub asset_source: Option<AssetSource>,
}

// TODO: hash for name and path separately
// TODO: check both for collisions
impl Assets {
    pub fn new() -> Self {
        let (send, recv) = std::sync::mpsc::channel::<Vec<LoadRequest>>();

        let current_queue = Arc::new(Mutex::new(None::<TextureLoadQueue>));
        let current_queue_inner = current_queue.clone();

        let image_map = Arc::new(Mutex::new(HashMap::new()));
        let image_map_inner = image_map.clone();

        std::thread::spawn(move || {
            while let Ok(texture_queue) = recv.recv() {
                // *queue.lock() = None;
                let iter = texture_queue.into_par_iter();

                let texture_queue: Vec<_> = iter
                    .filter_map(|request| {
                        let image = image::load_from_memory(&request.bytes);

                        match image {
                            Ok(image) => {
                                image_map_inner
                                    .lock()
                                    .insert(request.handle, image.clone());

                                Some(LoadedImage {
                                    path: request.path,
                                    handle: request.handle,
                                    image,
                                })
                            }
                            Err(err) => {
                                error!(
                                    "Failed to load {} ... {}",
                                    request.path, err
                                );
                                None
                            }
                        }
                    })
                    .collect();

                // let texture_queue = texture_queue
                //     .into_iter()
                //     .filter_map(|(path, handle, image)| {
                //         if let Some(image) = image {
                //             if image.width() == 0 || image.height() == 0 {
                //                 error!("Image {} has 0 width or height", path);
                //                 None
                //             } else {
                //                 image_map_inner
                //                     .lock()
                //                     .insert(handle, image.clone());
                //                 Some((path, handle, image.clone()))
                //             }
                //         } else {
                //         }
                //     })
                //     .collect_vec();

                let mut queue = current_queue_inner.lock();

                if let Some(queue) = queue.as_mut() {
                    queue.extend(texture_queue);
                } else {
                    *queue = Some(texture_queue);
                }
            }
        });

        let (tx_sound, rx_sound) =
            std::sync::mpsc::channel::<LoadSoundRequest>();

        let sounds = Arc::new(Mutex::new(HashMap::new()));
        let sounds_inner = sounds.clone();

        std::thread::spawn(move || {
            let pool = rayon::ThreadPoolBuilder::new().build().unwrap();

            while let Ok(item) = rx_sound.recv() {
                let sounds = sounds_inner.clone();

                pool.install(move || {
                    // TODO: do this properly
                    let settings = if item.path.contains("music") {
                        StaticSoundSettings::new().loop_region(..)
                    } else {
                        StaticSoundSettings::default()
                    };

                    match StaticSoundData::from_cursor(
                        std::io::Cursor::new(item.bytes),
                        settings,
                    ) {
                        Ok(sound) => {
                            trace!("Sound {}", item.path);
                            sounds.lock().insert(item.handle, sound);
                        }
                        Err(err) => {
                            error!(
                                "Failed to parse sound at {}: {:?}",
                                item.path, err
                            );
                        }
                    }
                });
            }
        });

        Self {
            texture_send: Arc::new(Mutex::new(send)),
            sound_send: Arc::new(Mutex::new(tx_sound)),

            textures: Default::default(),
            texture_load_queue: Default::default(),
            texture_image_map: image_map,

            sound_ids: HashMap::default(),
            sounds,
            sound_handles: HashMap::default(),
            sound_groups: HashMap::default(),

            thread_pool: rayon::ThreadPoolBuilder::new().build().unwrap(),

            current_queue,

            sound_load_queue: vec![],

            asset_source: None,
        }
    }

    pub fn process_load_queue(&mut self) {
        let _span = span!("process_load_queue");

        let loaded_textures = self
            .texture_image_map
            .lock()
            .keys()
            .cloned()
            .collect::<HashSet<_>>();

        let asset_source = self
            .asset_source
            .as_ref()
            .expect("AssetSource must be initialized by the game");

        let load_path_queue = self
            .texture_load_queue
            .drain(..)
            .filter(|(key, _relative_path)| {
                !loaded_textures.contains(&texture_id_unchecked(key))
            })
            .map(|(key, relative_path)| {
                let handle = texture_id_unchecked(&key);

                self.textures.insert(key, handle);

                if cfg!(feature = "ci-release") {
                    info!("Embedded texture {}", relative_path);

                    // let file = dir.get_file(&path);
                    // queue_load_texture_from_bytes(&path, file.contents()).unwrap()
                    // let contents = std::fs::read(&relative_path);
                    let file = asset_source
                        .dir
                        .get_file(&relative_path)
                        .unwrap_or_else(|| {
                            panic!("Failed to load {}", relative_path);
                        });

                    (relative_path, handle, Ok(file.contents().to_vec()))
                } else {
                    let absolute_path =
                        (asset_source.base_path)(&relative_path);

                    info!(
                        "File texture: {} ... {}",
                        relative_path, absolute_path
                    );

                    let absolute_path = std::path::Path::new(&absolute_path)
                        .canonicalize()
                        .unwrap()
                        .to_string_lossy()
                        .to_string();

                    trace!("Loading absolute path {}", absolute_path);

                    let contents = std::fs::read(absolute_path);

                    contents.as_ref().unwrap();

                    (relative_path, handle, contents)
                }
            })
            .filter_map(|(relative_path, handle, data)| {
                if let Ok(data) = data {
                    Some(LoadRequest {
                        path: relative_path,
                        handle,
                        bytes: data,
                    })
                } else {
                    error!("Error loading {}", relative_path);
                    None
                }
            })
            .collect_vec();

        let texture_queue = load_path_queue;

        self.texture_send.lock().send(texture_queue).log_err();
    }

    pub fn process_sound_queue(&mut self) {
        // let load_path_queue = self
        //     .sound_load_queue
        //     .drain(..)
        //     .filter(|path| !self.sounds.contains_key(&Sound::from_path(&path)))
        //     // .map(|path| (path.clone(), std::fs::read(&path)))
        //     .collect_vec();

        // let load_byte_queue = self
        //     .sound_load_bytes_queue
        //     .drain(..)
        //     .filter(|(path, _)| {
        //         !self.sounds.contains_key(&Sound::from_path(&path))
        //     })
        //     .map(|(path, bytes)| (path, Ok(bytes)))
        //     .collect_vec();

        let asset_source = self
            .asset_source
            .as_ref()
            .expect("AssetSource must be initialized by the game");

        for (key, relative_path) in self.sound_load_queue.drain(..) {
            let handle = Sound::from_path(&key);

            if self.sounds.lock().contains_key(&handle) {
                continue;
            }

            self.sound_ids.insert(key.to_string(), handle);

            let item = if cfg!(feature = "ci-release") {
                info!("Embedded Sound {}", relative_path);

                let file = asset_source
                    .dir
                    .get_file(&relative_path)
                    .unwrap_or_else(|| {
                        panic!("Failed to load {}", relative_path);
                    });

                LoadSoundRequest {
                    path: relative_path,
                    handle,
                    bytes: file.contents().to_vec(),
                }
            } else {
                info!("File Sound: {}", relative_path);
                let absolute_path = (asset_source.base_path)(&relative_path);

                let absolute_path = std::path::Path::new(&absolute_path)
                    .canonicalize()
                    .unwrap()
                    .to_string_lossy()
                    .to_string();

                trace!("Loading absolute path {}", absolute_path);

                let contents = std::fs::read(absolute_path).unwrap();

                LoadSoundRequest {
                    path: relative_path,
                    handle,
                    bytes: contents,
                }
            };

            self.sound_send.lock().send(item).log_err();
        }

        // let sound_queue = self
        //     .sound_load_queue
        //     .drain(..)
        //     .filter(|(key, _relative_path)| {
        //         !self.sounds.contains_key(&Sound::from_path(key))
        //     })
        //     .map(|(key, relative_path)| {
        //         let handle = Sound::from_path(&key);
        //
        //         self.sound_ids.insert(key.to_string(), handle);
        //
        //         if cfg!(any(feature = "ci-release", target_arch = "wasm32")) {
        //             info!("Embedded Sound {}", relative_path);
        //
        //             let file = asset_source
        //                 .dir
        //                 .get_file(&relative_path)
        //                 .unwrap_or_else(|| {
        //                     panic!("Failed to load {}", relative_path);
        //                 });
        //
        //             (relative_path, handle, Ok(file.contents().to_vec()))
        //         } else {
        //             info!("File Sound: {}", relative_path);
        //             let absolute_path =
        //                 (asset_source.base_path)(&relative_path);
        //
        //             let absolute_path = std::path::Path::new(&absolute_path)
        //                 .canonicalize()
        //                 .unwrap()
        //                 .to_string_lossy()
        //                 .to_string();
        //
        //             trace!("Loading absolute path {}", absolute_path);
        //
        //             let contents = std::fs::read(absolute_path);
        //
        //             contents.as_ref().unwrap();
        //
        //             (relative_path, handle, contents)
        //         }
        //     })
        //     .filter_map(|(relative_path, handle, data)| {
        //         if let Ok(data) = data {
        //             Some(LoadSoundRequest {
        //                 path: relative_path,
        //                 handle,
        //                 bytes: data,
        //             })
        //         } else {
        //             error!("Error loading {}", relative_path);
        //             None
        //         }
        //     })
        //     .collect_vec();

        // for item in sound_queue.into_iter() {
        //     let settings = if item.path.contains("music") {
        //         StaticSoundSettings::new()
        //             .loop_behavior(Some(LoopBehavior { start_position: 0.0 }))
        //     } else {
        //         StaticSoundSettings::default()
        //     };
        //
        //     info!("Loading sound {}", item.path);
        //
        //     match StaticSoundData::from_cursor(
        //         std::io::Cursor::new(item.bytes),
        //         settings,
        //     ) {
        //         Ok(sound) => {
        //             info!("Loaded {}", item.path);
        //             self.sounds.insert(item.handle, sound);
        //         }
        //         Err(err) => {
        //             error!("Failed to parse sound at {}: {:?}", item.path, err);
        //         }
        //     }
        // }
    }

    pub fn handle_name(handle: TextureHandle) -> Option<String> {
        ASSETS.borrow().textures.iter().find_map(|(k, v)| {
            if *v == handle {
                Some(k.clone())
            } else {
                None
            }
        })
    }

    pub fn insert_handle(&mut self, name: &str, handle: TextureHandle) {
        self.textures.insert(name.to_string(), handle);
    }

    pub fn get_texture(&self, key: &str) -> TextureHandle {
        match self.textures.get(key) {
            Some(val) => *val,
            None => {
                error!("Missing {}", key);
                error!("");
                error!("Available:");
                for key in self.textures.keys().sorted() {
                    error!("   {}", key);
                }

                panic!("Unable to load texture {}", key);
            }
        }
    }

    pub fn image_size(handle: TextureHandle) -> Option<UVec2> {
        let assets = ASSETS.borrow();
        let image_map = assets.texture_image_map.lock();

        let image = image_map.get(&handle)?;

        Some(uvec2(image.width(), image.height()))
    }

    pub fn load_image_data(
        path: &str,
        texture: TextureHandle,
    ) -> Option<DynamicImage> {
        let assets = ASSETS.borrow();

        let image_map = assets.texture_image_map.lock();

        let Some(image) = image_map.get(&texture) else {
            let mut messages = vec![];

            if get_time() > 5.0 {
                println!("Loaded image map:\n");

                for (id, _) in image_map.iter() {
                    if let Some((path, _)) =
                        ASSETS.borrow().textures.iter().find(|x| x.1 == id)
                    {
                        messages.push(format!("{}: {:?}", path, id));
                    } else {
                        messages.push(format!("no path: {:?}", id));
                    };
                }

                messages.sort();

                for message in messages.into_iter() {
                    println!("{}", message);
                }

                println!("Failed to load image map {}", path);
                println!();
            }

            return None;
        };

        Some(image.clone())
        // let image = texture.get_texture_data();
    }

    pub fn error_loading_image(path: &str) {
        println!("Loaded textures:\n");

        let mut messages = vec![];

        for (path, id) in ASSETS.borrow().textures.iter() {
            messages.push(format!("{}: {:?}", path, id));
        }

        messages.sort();

        for message in messages.into_iter() {
            println!("{}", message);
        }

        println!("Failed to load textures image {}", path);
    }
}

pub fn load_multiple_sounds(pairs: &Vec<(String, String)>) {
    for (key, relative_path) in pairs {
        ASSETS
            .borrow_mut()
            .sound_load_queue
            .push((key.clone(), relative_path.clone()));
    }
}

pub fn load_multiple_textures(pairs: &[(String, String)]) {
    let mut assets = ASSETS.borrow_mut();

    for (key, relative_path) in pairs.iter() {
        assets.texture_load_queue.push((key.clone(), relative_path.clone()));
    }
}
