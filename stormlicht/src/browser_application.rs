use core::BrowsingContext;
use std::process::ExitCode;

use url::URL;

const INITIAL_WIDTH: u16 = 800;
const INITIAL_HEIGHT: u16 = 600;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
enum RepaintRequired {
    #[default]
    Yes,
    No,
}

pub struct BrowserApplication {
    view_buffer: math::Bitmap<u32>,
    graphics_context: Option<softbuffer::GraphicsContext>,
    size: (u16, u16),
    repaint_required: RepaintRequired,
    composition: render::Composition,
    window_handle: glazier::WindowHandle,
    _browsing_context: BrowsingContext,
}

impl glazier::WinHandler for BrowserApplication {
    fn connect(&mut self, handle: &glazier::WindowHandle) {
        let graphics_context = unsafe { softbuffer::GraphicsContext::new(handle, handle) }.expect("Failed to connect to softbuffer graphics context");
        self.window_handle = handle.clone();
        self.graphics_context = Some(graphics_context);
    }

    fn prepare_paint(&mut self) {
        if self.repaint_required == RepaintRequired::Yes {
            self.window_handle.invalidate();
        }
    }

    fn paint(&mut self, _invalid: &glazier::Region) {
        self.view_buffer.clear(math::Color::WHITE.into());
        self.composition.render_to(&mut self.view_buffer);

        if let Some(graphics_context) = &mut self.graphics_context {
            graphics_context.set_buffer(self.view_buffer.data(), self.size.0, self.size.1);
        }
        self.repaint_required = RepaintRequired::No;
    }

    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn size(&mut self, size: glazier::kurbo::Size) {
        let width = size.width.ceil() as u16 * 2;
        let height = size.height.ceil() as u16 * 2;
        self.size = (width, height);
        self.view_buffer.resize(width as usize, height as usize);
        self.repaint_required = RepaintRequired::Yes;
    }

    fn request_close(&mut self) {
        self.window_handle.close();
        glazier::Application::global().quit();
    }
}

impl BrowserApplication {
    pub fn run(url: Option<&str>) -> ExitCode {
        let font = font::Font::default();
        let d = font.compute_rendered_width("Font test", 200.);
        let mut composition = render::Composition::default();

        composition
            .get_or_insert_layer(1)
            .with_source(render::Source::Solid(math::Color::BLUE))
            .with_outline(render::Path::rect(
                math::Vec2D::new(50., 50.),
                math::Vec2D::new(50. + d, 250.),
            ));

        composition
            .get_or_insert_layer(2)
            .with_source(render::Source::Solid(math::Color::BLACK))
            .text(
                "Font test",
                font::Font::default(),
                200.,
                math::Vec2D::new(50., 50.),
            );

        let browsing_context = match url {
            Some(url) => {
                let url = match URL::from_user_input(url) {
                    Ok(parsed_url) => parsed_url,
                    Err(error) => {
                        log::error!("Failed to parse {url:?} as a URL: {error:?}");
                        return ExitCode::FAILURE;
                    }
                };

                match BrowsingContext::load(&url) {
                    Ok(context) => context,
                    Err(error) => {
                        log::error!("Failed to load {}: {error:?}", url.to_string());
                        return ExitCode::FAILURE;
                    }
                }
            },
            None => {
                // FIXME: default url
                BrowsingContext
            },
        };

        let application = Self {
            view_buffer: math::Bitmap::new(INITIAL_WIDTH as usize, INITIAL_HEIGHT as usize),
            graphics_context: None,
            size: (INITIAL_WIDTH, INITIAL_HEIGHT),
            repaint_required: RepaintRequired::Yes,
            composition,
            window_handle: glazier::WindowHandle::default(),
            _browsing_context: browsing_context,
        };

        let app = match glazier::Application::new() {
            Ok(app) => app,
            Err(error) => {
                log::error!("Failed to initialize application: {error:?}");
                return ExitCode::FAILURE;
            }
        };

        let window_or_error = glazier::WindowBuilder::new(app.clone())
            .resizable(true)
            .size(((INITIAL_WIDTH / 2) as f64, (INITIAL_HEIGHT / 2) as f64).into())
            .handler(Box::new(application))
            .title("Browser")
            .build()
            ;
        match window_or_error {
            Ok(window) => {
                window.show();
                app.run(None);
                ExitCode::SUCCESS
            },
            Err(error) => {
                log::error!("Failed to create application window: {error:?}");
                ExitCode::FAILURE
            }
        }
    }
}
