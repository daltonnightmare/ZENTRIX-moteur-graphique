//! Les maillages sur le GPU.
//!
//! Deux types, deux mondes :
//!
//! - [`MeshData`] vit en **mémoire CPU** (de simples `Vec`, dans
//!   `zentrix-core`). On peut le créer, le modifier, le tester sans GPU.
//! - [`Mesh`] vit en **mémoire GPU** (des buffers). Il est créé à partir d'un
//!   `MeshData` puis rangé dans le `Renderer`, qui rend en échange un
//!   [`MeshHandle`].

use zentrix_core::MeshData;
use zentrix_rhi::wgpu;
use zentrix_rhi::wgpu::util::DeviceExt;

/// Une référence légère vers un maillage stocké dans le `Renderer`.
///
/// C'est un simple numéro (`usize`), donc `Copy` et gratuit à dupliquer :
/// 1000 cubes peuvent partager le même `MeshHandle`, la géométrie n'existe
/// qu'une fois sur le GPU. Le champ est privé : on obtient normalement un
/// handle avec `Renderer::add_mesh`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MeshHandle(pub(crate) usize);

impl MeshHandle {
    /// Fabrique un handle à partir d'un numéro, **sans vérification**.
    ///
    /// Réservé aux tests et aux outils, qui ont besoin de handles sans GPU
    /// (par exemple les tests de `zentrix-scene`). Dessiner avec un handle
    /// qui ne correspond à aucun maillage chargé provoque un panic.
    #[doc(hidden)]
    pub const fn from_index(index: usize) -> Self {
        Self(index)
    }
}

/// Un maillage chargé sur le GPU.
pub(crate) struct Mesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub index_count: u32,
}

impl Mesh {
    /// Copie un [`MeshData`] dans deux buffers GPU.
    pub fn upload(device: &wgpu::Device, data: &MeshData) -> Self {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Mesh Vertex Buffer"),
            contents: bytemuck::cast_slice(&data.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Mesh Index Buffer"),
            contents: bytemuck::cast_slice(&data.indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        Self {
            vertex_buffer,
            index_buffer,
            index_count: data.indices.len() as u32,
        }
    }
}
