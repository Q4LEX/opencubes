use ash::{vk::SurfaceKHR, Entry};
use winit::window::Window;

use super::instance::Instance;

pub struct Surface {
    pub inner: SurfaceKHR,
    pub loader: ash::extensions::khr::Surface,
}

impl Surface {
    pub fn new(entry: &Entry, instance: &Instance, window: &Window) -> Self {
        let inner =
            unsafe { ash_window::create_surface(entry, &instance.inner, window, None).unwrap() };
        let loader = ash::extensions::khr::Surface::new(entry, &instance.inner);

        Surface { inner, loader }
    }
}

impl Drop for Surface {
    fn drop(&mut self) {
        unsafe {
            self.loader.destroy_surface(self.inner, None);
        }
    }
}
