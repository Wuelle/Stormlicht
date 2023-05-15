//! [Layer] management

use std::collections::{hash_map::Iter, HashMap};

use crate::{Buffer, Layer};

/// Manages all the different [Layers](Layer) that should be rendered.
///
/// Generally, there should never be a need to create more than one [Composition].
#[derive(Debug, Clone, Default)]
pub struct Composition {
    layers: HashMap<u16, Layer>,
}

impl Composition {
    /// Tries to retrieve the [Layer] at the given index in the composition.
    ///
    /// If there is no layer at the current index, a default layer is created and
    /// returned.
    pub fn get_or_insert_layer(&mut self, at_index: u16) -> &mut Layer {
        self.layers.entry(at_index).or_insert_with(Layer::default)
    }

    pub fn layers(&self) -> Iter<'_, u16, Layer> {
        self.layers.iter()
    }

    pub fn render_to(&mut self, buffer: &mut Buffer) {
        // Draw all the layers, in order
        let mut keys: Vec<u16> = self.layers.keys().copied().collect();
        keys.sort();

        for key in keys {
            let layer = self
                .layers
                .get_mut(&key)
                .expect("Every key returned by layers.keys() should be valid");

            layer.render_to(buffer);
        }
    }
}