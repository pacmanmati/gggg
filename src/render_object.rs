use nalgebra::Matrix4;

use crate::{
    geometry::{BasicGeometry, Geometry},
    instance::{BasicInstance, InstanceData},
    material::{BasicMaterial, Material},
    pipeline::PipelineHandle,
    render::{MeshHandle, Render, TextureHandle},
};

// A RenderObject defines how instance data is obtained from the renderer, mesh and material data for a particular pipeline.
// E.g. A RenderObject's material might define a texture for the mesh to use.
// api?
// a renderobject describes how a material + mesh create instance data
// it doesn't necessarily map 1-1 with a pipeline.
// maybe we need to specify which pipeline a mesh needs to be drawn with when we submit it
// we ask the pipeline for its renderobject implementor which accepts that mesh and material type, giving us the instance data.
// this feels overly complex but i think it gives us the flexibility we need to define our own materials in code.
//
// what does a renderobject give us that a mesh doesn't? - mesh is a struct not a trait i guess
pub trait RenderObject {
    type InstanceType: InstanceData;
    type GeometryType: Geometry + 'static;
    type MaterialType: Material + 'static;
    fn instance(&self, render: &Render) -> Self::InstanceType;
    fn pipeline_handle(&self) -> PipelineHandle;
    fn mesh_handle(&self) -> MeshHandle;
    fn boxed(self) -> BoxedRenderObject<Self::GeometryType, Self::InstanceType, Self::MaterialType>
    // holy shit it works
    where
        Self: Sized + 'static,
    {
        BoxedRenderObject(Box::new(self))
    }
}

// problem: we want to access the original render object's methods but we cannot store it. what do we do?
// we cannot store Box<dyn RenderObject<...>> because we end up having to create a version of RenderObject with trait object associated types (which is what we need this struct for!)
// we cannot call boxed recursively
// we could store function pointers (lifetime / borrow issues? - we don't have the underlying data to refer to)
// we can honestly store some of the data. eg. pipelinehandle, meshhandle. instance() is the only real problem.
// can we store it as any? and downcast it? or can it be generic instead? - if it's a generic the boxedrenderobject requires generics too. is that a problem?
// generics work!
pub struct BoxedRenderObject<G, I, M>(
    Box<dyn RenderObject<GeometryType = G, InstanceType = I, MaterialType = M>>,
);

// pub struct BoxedRenderObject(
//     Box<
//         dyn RenderObject<
//             GeometryType = Box<dyn Geometry>,
//             InstanceType = Box<dyn InstanceData>,
//             MaterialType = Box<dyn Material>,
//         >,
//     >,
// );

impl<G: Geometry + 'static, I: InstanceData + 'static, M: Material + 'static> RenderObject
    for BoxedRenderObject<G, I, M>
{
    type InstanceType = Box<dyn InstanceData>;

    type GeometryType = Box<dyn Geometry>;

    type MaterialType = Box<dyn Material>;

    fn instance(
        &self,
        render: &Render,
        // mesh: Mesh<Self::GeometryType, Self::MaterialType>,
    ) -> Self::InstanceType {
        Box::new(self.0.as_ref().instance(render))
    }

    fn pipeline_handle(&self) -> PipelineHandle {
        self.0.as_ref().pipeline_handle()
    }

    fn mesh_handle(&self) -> MeshHandle {
        self.0.as_ref().mesh_handle()
    }
}

pub struct BasicRenderObject {
    // TODO: maybe move pipeline handle into material?
    pub pipeline_handle: PipelineHandle,
    pub mesh_handle: MeshHandle,
    pub transform: Matrix4<f32>,
    // TODO: move texture into material. replace with material_handle
    pub texture_handle: TextureHandle,
}

impl RenderObject for BasicRenderObject {
    type InstanceType = BasicInstance;

    type GeometryType = BasicGeometry;

    type MaterialType = BasicMaterial;

    fn instance(&self, render: &Render) -> Self::InstanceType {
        let atlas_coords = render.get_atlas_coords_for_texture(self.texture_handle);
        // println!("{:?}", atlas_coords);
        BasicInstance {
            transform: self.transform,
            atlas_coords: atlas_coords.into(),
        }
    }

    fn pipeline_handle(&self) -> PipelineHandle {
        // TODO: we need to create a basic pipeline whose pipeline handle can be assumed
        // e.g. we ensure that the basic pipeline is added first and its index is always 0
        self.pipeline_handle
    }

    fn mesh_handle(&self) -> MeshHandle {
        self.mesh_handle
    }
}
