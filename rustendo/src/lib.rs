use rustendo_lib::nes::Nes;
use std::cell::RefCell;
use std::f64;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

mod utils;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}

fn request_animation_frame(f: &Closure<dyn FnMut(f64)>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

#[wasm_bindgen]
pub fn render() {
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id("rustendo-canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| ())
        .unwrap();

    let context = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .unwrap();

    let f = Rc::new(RefCell::new(None));
    let g = Rc::clone(&f);

    let mut prev_timestamp = 0.0;
    *g.borrow_mut() = Some(Closure::wrap(Box::new(move |timestamp| {
        web_sys::console::log(&js_sys::Array::of1(&JsValue::from_f64(
            timestamp - prev_timestamp,
        )));
        draw(&context, &canvas);
        request_animation_frame(f.borrow().as_ref().unwrap());
        prev_timestamp = timestamp;
    }) as Box<dyn FnMut(f64)>));

    request_animation_frame(g.borrow().as_ref().unwrap());
}

fn draw(context: &web_sys::CanvasRenderingContext2d, canvas: &web_sys::HtmlCanvasElement) {
    let image_data = context
        .get_image_data(0.0, 0.0, canvas.width().into(), canvas.height().into())
        .expect("failed to get ImageData");

    let mut data = image_data.data();

    for y in 0..image_data.height() {
        for x in 0..image_data.width() {
            let color: (u8, u8, u8, u8) = if js_sys::Math::random() < 0.5 {
                (0, 0, 0, 255)
            } else {
                (255, 255, 255, 255)
            };

            let indices = get_color_indices_for_coord(x, y, canvas.width());
            data[indices.0] = color.0;
            data[indices.1] = color.1;
            data[indices.2] = color.2;
            data[indices.3] = color.3;
        }
    }

    let image_data = web_sys::ImageData::new_with_u8_clamped_array(
        wasm_bindgen::Clamped(&mut data),
        canvas.width(),
    )
    .expect("could not create image data");
    context
        .put_image_data(&image_data, 0.0, 0.0)
        .expect("could not put image data");
}

fn get_color_indices_for_coord(x: u32, y: u32, width: u32) -> (usize, usize, usize, usize) {
    let x = x as usize;
    let y = y as usize;
    let width = width as usize;
    let red_index = y * (width * 4) + x * 4;
    (red_index, red_index + 1, red_index + 2, red_index + 3)
}

trait DrawPixel {
    fn draw_pixel(&self, x: f64, y: f64, color: &str);
}

impl DrawPixel for web_sys::CanvasRenderingContext2d {
    fn draw_pixel(&self, x: f64, y: f64, color: &str) {
        self.set_fill_style(&JsValue::from_str(color));
        self.fill_rect(x, y, 1.0, 1.0);
    }
}
