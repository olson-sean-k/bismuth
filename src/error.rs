use super::cube;
use super::framework;
use super::render;

error_chain! {
    links {
        Cube(cube::error::CubeError, cube::error::ErrorKind);
        Framework(framework::error::FrameworkError, framework::error::ErrorKind);
        Render(render::error::RenderError, render::error::ErrorKind);
    }
}
