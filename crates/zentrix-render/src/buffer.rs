//! [`GrowableBuffer`] : un buffer GPU réécrit à chaque image, qui s'agrandit
//! quand les données ne tiennent plus.

use zentrix_rhi::wgpu;

/// Un buffer dont le contenu change à chaque image (matrices des instances,
/// lignes de debug...).
///
/// Un buffer GPU a une taille **fixe**. Quand les données dépassent, on en
/// crée un plus grand — au moins deux fois plus grand, pour ne pas le recréer
/// à chaque nouvel élément (même stratégie qu'un `Vec`). On ne le réduit
/// jamais : la mémoire sera sans doute réutilisée à l'image suivante.
pub(crate) struct GrowableBuffer {
    buffer: wgpu::Buffer,
    /// Taille actuelle en octets.
    capacity: wgpu::BufferAddress,
    label: &'static str,
    usage: wgpu::BufferUsages,
}

impl GrowableBuffer {
    /// `usage` reçoit automatiquement `COPY_DST`, nécessaire pour `write`.
    pub fn new(
        device: &wgpu::Device,
        label: &'static str,
        usage: wgpu::BufferUsages,
        initial_capacity: wgpu::BufferAddress,
    ) -> Self {
        let usage = usage | wgpu::BufferUsages::COPY_DST;
        Self {
            buffer: create(device, label, usage, initial_capacity),
            capacity: initial_capacity,
            label,
            usage,
        }
    }

    /// Copie `data` au début du buffer, en l'agrandissant si besoin.
    pub fn write(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, data: &[u8]) {
        let needed = data.len() as wgpu::BufferAddress;
        if needed == 0 {
            return;
        }
        if needed > self.capacity {
            self.capacity = needed.next_power_of_two();
            self.buffer = create(device, self.label, self.usage, self.capacity);
            log::debug!("{} agrandi à {} octets", self.label, self.capacity);
        }
        queue.write_buffer(&self.buffer, 0, data);
    }

    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }
}

fn create(
    device: &wgpu::Device,
    label: &str,
    usage: wgpu::BufferUsages,
    size: wgpu::BufferAddress,
) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size,
        usage,
        mapped_at_creation: false,
    })
}
