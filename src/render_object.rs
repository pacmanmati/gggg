use crate::{
    geometry::Geometry,
    material::Material,
    pipeline::PipelineHandle,
    render::{InstanceData, Mesh, Render},
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
    fn instance_data(
        &self,
        render: &Render,
        mesh: Mesh<Self::GeometryType, Self::MaterialType>,
    ) -> Self::InstanceType;
    fn pipeline_handle(&self) -> PipelineHandle;
    fn geometry() -> Self::GeometryType;
    fn material() -> Self::MaterialType;
}
