use winit::event::{ElementState, Event, WindowEvent, DeviceEvent, DeviceId};
use winit::dpi::PhysicalPosition;
use std::collections::BTreeMap;

use crate::WindowId;
use crate::event::MouseButtonState;

#[derive(Default)]
pub struct MouseCache {
	mouse_buttons: BTreeMap<DeviceId, MouseButtonState>,
	mouse_position: BTreeMap<(WindowId, DeviceId), PhysicalPosition<f64>>,
	mouse_previous_position: BTreeMap<(WindowId, DeviceId), PhysicalPosition<f64>>,
}

impl MouseCache {
	pub fn get_position(&self, window_id: WindowId, device_id: DeviceId) -> Option<PhysicalPosition<f64>> {
		self.mouse_position.get(&(window_id, device_id)).copied()
	}

	pub fn get_previous_position(&self, window_id: WindowId, device_id: DeviceId) -> Option<PhysicalPosition<f64>> {
		self.mouse_previous_position.get(&(window_id, device_id)).copied()
	}

	pub fn get_buttons(&self, device_id: DeviceId) -> Option<&MouseButtonState> {
		self.mouse_buttons.get(&device_id)
	}

	pub fn handle_event(&mut self, event: &Event<()>) {
		match event {
			Event::WindowEvent { window_id, event } => self.handle_window_event(*window_id, event),
			Event::DeviceEvent { device_id, event } => self.handle_device_event(*device_id, event),
			_ => (),
		}
	}

	fn handle_window_event(&mut self, window_id: WindowId, event: &WindowEvent) {
		match event {
			WindowEvent::MouseInput { device_id, button, state, .. } => {
				let buttons = self.mouse_buttons.entry(*device_id).or_default();
				buttons.set_pressed((*button).into(), *state == ElementState::Pressed);
			},
			WindowEvent::CursorMoved { device_id, position, .. } => {
				let cached_position = self.mouse_position.entry((window_id, *device_id)).or_insert_with(|| [0.0, 0.0].into());
				let cached_previous_position = self.mouse_previous_position.entry((window_id, *device_id)).or_insert_with(|| [0.0, 0.0].into());
				*cached_previous_position = *cached_position;
				*cached_position = *position;
			},
			_ => {},
		}
	}

	fn handle_device_event(&mut self, device_id: DeviceId, event: &DeviceEvent) {
		if let DeviceEvent::Removed = event {
			self.remove_device(device_id)
		}
	}

	fn remove_device(&mut self, device_id: DeviceId) {
		self.mouse_buttons.remove(&device_id);
		let keys: Vec<_> = self.mouse_position.keys().filter(|(_, x)| *x == device_id).copied().collect();
		for key in &keys {
			self.mouse_position.remove(&key);
		}
		let keys: Vec<_> = self.mouse_previous_position.keys().filter(|(_, x)| *x == device_id).copied().collect();
		for key in &keys {
			self.mouse_previous_position.remove(&key);
		}
	}
}
