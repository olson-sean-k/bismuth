use cube;
use framework;
use render;

error_chain! {
    links {
        Cube(cube::error::CubeError, cube::error::ErrorKind);
        Framework(framework::error::FrameworkError, framework::error::ErrorKind);
        Render(render::error::RenderError, render::error::ErrorKind);
    }
}
