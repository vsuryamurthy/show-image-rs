use crate::Color;
use crate::ContextHandle;
use crate::EventHandlerOutput;
use crate::Image;
use crate::WindowId;
use crate::backend::util::Texture;
use crate::backend::util::UniformsBuffer;
use crate::error::InvalidWindowIdError;
use crate::event::WindowEvent;

pub struct Window<UserEvent: 'static> {
	pub(crate) window: winit::window::Window,
	pub(crate) options: WindowOptions,
	pub(crate) surface: wgpu::Surface,
	pub(crate) swap_chain: wgpu::SwapChain,
	pub(crate) uniforms: UniformsBuffer<WindowUniforms>,
	pub(crate) image: Option<Texture>,
	pub(crate) event_handlers: Vec<Box<dyn FnMut(WindowHandle<UserEvent>, &mut crate::event::WindowEvent) -> EventHandlerOutput>>,
}

pub struct WindowHandle<'a, UserEvent: 'static> {
	context_handle: ContextHandle<'a, UserEvent>,
	window_id: WindowId,
}

impl<'a, UserEvent> WindowHandle<'a, UserEvent> {
	pub(crate) fn new(context_handle: ContextHandle<'a, UserEvent>, window_id: WindowId) -> Self {
		Self { context_handle, window_id }
	}

	pub fn id(&self) -> WindowId {
		self.window_id
	}

	pub fn context_handle(&mut self) -> &mut ContextHandle<'a, UserEvent> {
		&mut self.context_handle
	}

	pub fn destroy(mut self) -> Result<(), InvalidWindowIdError> {
		self.context_handle.destroy_window(self.window_id)
	}

	pub fn set_visible(&mut self, visible: bool) -> Result<(), InvalidWindowIdError> {
		self.context_handle.set_window_visible(self.window_id, visible)
	}

	pub fn set_image(&mut self, name: impl AsRef<str>, image: &Image) -> Result<(), InvalidWindowIdError> {
		self.context_handle.set_window_image(self.window_id, name.as_ref(), image)
	}

	pub fn add_event_handler<F>(&mut self, handler: F) -> Result<(), InvalidWindowIdError>
	where
		F: 'static + FnMut(WindowHandle<UserEvent>, &mut WindowEvent) -> EventHandlerOutput,
	{
		self.context_handle.add_window_event_handler(self.window_id, handler)
	}

	pub fn add_boxed_event_handler(
		&mut self,
		handler: Box<dyn FnMut(WindowHandle<UserEvent>, &mut WindowEvent) -> EventHandlerOutput>,
	) -> Result<(), InvalidWindowIdError> {
		self.context_handle.add_boxed_window_event_handler(self.window_id, handler)
	}
}

#[derive(Debug, Clone)]
pub struct WindowOptions {
	/// Preserve the aspect ratio of the image when scaling.
	pub preserve_aspect_ratio: bool,

	/// The background color for the window.
	///
	/// This is used to color areas without image data if `preserve_aspect_ratio` is true.
	pub background_color: Color,

	/// Create the window hidden.
	///
	/// The window can manually be made visible at a later time.
	pub start_hidden: bool,

	/// The initial size of the window in pixel.
	///
	/// This may be ignored by a window manager.
	pub size: Option<[u32; 2]>,

	/// If true allow the window to be resized.
	///
	/// This may be ignored by a window manager.
	pub resizable: bool,
}

impl Default for WindowOptions {
	fn default() -> Self {
		Self {
			preserve_aspect_ratio: true,
			background_color: Color::BLACK,
			start_hidden: false,
			size: None,
			resizable: true,
		}
	}
}

impl WindowOptions {
	/// Preserve the aspect ratio of displayed images, or not.
	///
	/// This function consumes and returns `self` to allow daisy chaining.
	pub fn set_preserve_aspect_ratio(mut self, preserve_aspect_ratio: bool) -> Self {
		self.preserve_aspect_ratio = preserve_aspect_ratio;
		self
	}
	/// Set the background color of the window.
	///
	/// This function consumes and returns `self` to allow daisy chaining.
	pub fn set_background_color(mut self, background_color: Color) -> Self {
		self.background_color = background_color;
		self
	}

	/// Start the window hidden.
	///
	/// This function consumes and returns `self` to allow daisy chaining.
	pub fn set_start_hidden(mut self, start_hidden: bool) -> Self {
		self.start_hidden = start_hidden;
		self
	}

	/// Set the initial size of the window.
	///
	/// This property may be ignored by a window manager.
	///
	/// This function consumes and returns `self` to allow daisy chaining.
	pub fn set_size(mut self, size: [u32; 2]) -> Self {
		self.size = Some(size);
		self
	}

	/// Make the window resizable or not.
	///
	/// This property may be ignored by a window manager.
	///
	/// This function consumes and returns `self` to allow daisy chaining.
	pub fn set_resizable(mut self, resizable: bool) -> Self {
		self.resizable = resizable;
		self
	}
}

impl<UserEvent> Window<UserEvent> {
	pub fn id(&self) -> WindowId {
		self.window.id()
	}

	pub fn set_visible(&mut self, visible: bool) {
		self.window.set_visible(visible);
	}

	pub(crate) fn calculate_uniforms(&self) -> WindowUniforms {
		WindowUniforms {
			scale: self.calculate_scale(),
		}
	}

	fn calculate_scale(&self) -> [f32; 2] {
		if !self.options.preserve_aspect_ratio {
			[1.0, 1.0]
		} else if let Some(image) = &self.image {
			let image_size = [image.width() as f32, image.height() as f32];
			let window_size = [self.window.inner_size().width as f32, self.window.inner_size().height as f32];
			let ratios = [image_size[0] / window_size[0], image_size[1] / window_size[1]];

			if ratios[0] >= ratios[1] {
				[1.0, ratios[1] / ratios[0]]
			} else {
				[ratios[0] / ratios[1], 1.0]
			}
		} else {
			[1.0, 1.0]
		}
	}
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct WindowUniforms {
	pub scale: [f32; 2],
}

impl Default for WindowUniforms {
	fn default() -> Self {
		Self {
			scale: [1.0, 1.0],
		}
	}
}
