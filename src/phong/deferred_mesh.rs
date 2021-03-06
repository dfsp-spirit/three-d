
use crate::math::*;
use crate::core::*;
use crate::object::*;
use crate::phong::*;

pub struct PhongDeferredMesh {
    context: Context,
    pub name: String,
    mesh: Mesh,
    pub material: PhongMaterial
}

impl PhongDeferredMesh {

    pub fn new(context: &Context, cpu_mesh: &CPUMesh, material: &PhongMaterial) -> Result<Self, Error>
    {
        if cpu_mesh.normals.is_none() {
            Err(Error::FailedToCreateMesh {message:
              "Cannot create a mesh without normals. Consider calling compute_normals on the CPUMesh before creating the mesh.".to_string()})?
        }
        let mesh = Mesh::new(context, cpu_mesh)?;
        unsafe {MESH_COUNT += 1;}
        Ok(Self {
            context: context.clone(),
            name: cpu_mesh.name.clone(),
            mesh,
            material: material.clone()
        })
    }

    pub fn new_meshes(context: &Context, cpu_meshes: &[CPUMesh], materials: &[PhongMaterial]) -> Result<Vec<Self>, Error>
    {
        let mut meshes = Vec::new();
        for cpu_mesh in cpu_meshes {
            let material = cpu_mesh.material_name.as_ref().map(|material_name|
                materials.iter().filter(|m| &m.name == material_name).last()
                .map(|m| m.clone()).unwrap_or_else(|| PhongMaterial::default()))
                .unwrap_or_else(|| PhongMaterial::default());
            meshes.push(Self::new(context,cpu_mesh, &material)?);
        }
        Ok(meshes)
    }

    pub fn render_geometry(&self, render_states: RenderStates, viewport: Viewport, transformation: &Mat4, camera: &camera::Camera) -> Result<(), Error>
    {
        let program = match self.material.color_source {
            ColorSource::Color(_) => {
                unsafe {
                    if PROGRAM_COLOR.is_none()
                    {
                        PROGRAM_COLOR = Some(MeshProgram::new(&self.context, &format!("{}\n{}",
                                                             include_str!("shaders/deferred_objects_shared.frag"),
                                                             include_str!("shaders/colored_deferred.frag")))?);
                    }
                    PROGRAM_COLOR.as_ref().unwrap()
                }
            },
            ColorSource::Texture(_) => {
                unsafe {
                    if PROGRAM_TEXTURE.is_none()
                    {
                        PROGRAM_TEXTURE = Some(MeshProgram::new(&self.context, &format!("{}\n{}",
                                                             include_str!("shaders/deferred_objects_shared.frag"),
                                                             include_str!("shaders/textured_deferred.frag")))?);
                    }
                    PROGRAM_TEXTURE.as_ref().unwrap()
                }
            }
        };
        self.material.bind(program)?;
        self.mesh.render(program, render_states, viewport, transformation, camera)
    }
}

impl std::ops::Deref for PhongDeferredMesh {
    type Target = Mesh;

    fn deref(&self) -> &Mesh {
        &self.mesh
    }
}

impl Drop for PhongDeferredMesh {

    fn drop(&mut self) {
        unsafe {
            MESH_COUNT -= 1;
            if MESH_COUNT == 0 {
                PROGRAM_COLOR = None;
                PROGRAM_TEXTURE = None;
            }
        }
    }
}

static mut PROGRAM_COLOR: Option<MeshProgram> = None;
static mut PROGRAM_TEXTURE: Option<MeshProgram> = None;
static mut MESH_COUNT: u32 = 0;