use gfx;
use glutin;
use image;

error_chain! {
    types {
        RenderError, ErrorKind, ResultExt, Result;
    }
    foreign_links {
        Context(glutin::ContextError);
        Generic(gfx::CombinedError);
        Image(image::ImageError);
    }
}
