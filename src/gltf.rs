use std::{fmt::Debug, path::Path};

use anyhow::{anyhow, Result};

use crate::{
    geometry::{BasicGeometry, Vertex},
    material::BasicMaterial,
    render::Mesh,
};

pub fn load_mesh<P: AsRef<Path> + Debug + Copy>(
    path: P,
) -> Result<Vec<Mesh<BasicGeometry, BasicMaterial>>> {
    let gltf = easy_gltf::load(path).map_err(|_| anyhow!("Couldn't load gltf at {:?}", path))?;

    let models = gltf.iter().flat_map(|scene| &scene.models);

    let mut meshes = Vec::new();

    for model in models {
        let vertices = model
            .vertices()
            .iter()
            .map(|vertex| Vertex {
                pos: vertex.position.into(),
                uv: vertex.tex_coords.into(),
                normal: vertex.normal.into(),
            })
            .collect();

        let indices = model
            .indices()
            .map(|indices| indices.iter().map(|index| *index as u16).collect());

        meshes.push(Mesh {
            material: BasicMaterial {},
            geometry: BasicGeometry { vertices, indices },
        });
    }

    Ok(meshes)

    // Ok(BasicGeometry { vertices, indices })
}
