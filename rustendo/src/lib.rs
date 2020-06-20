use js_sys::Array;
use rustendo_lib::cartridge::Cartridge;
use rustendo_lib::nes::Nes;
use std::cell::RefCell;
use std::f64;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::{Clamped, JsCast};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, ImageData, Window};

mod utils;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

fn window() -> Window {
    web_sys::window().expect("no global `window` exists")
}

fn request_animation_frame(f: &Closure<dyn FnMut(f64)>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

static mut STOP_ANIMATION: bool = false;

fn continue_animation() -> bool {
    unsafe { !STOP_ANIMATION }
}

#[wasm_bindgen]
pub fn stop_animation() {
    unsafe { STOP_ANIMATION = true };
}

#[wasm_bindgen]
pub fn startup() {
    utils::set_panic_hook();
}

#[wasm_bindgen]
pub fn render(byte_array: js_sys::Uint8Array) {
    unsafe { STOP_ANIMATION = false };

    let nes = Nes::new();
    let vec = byte_array.to_vec();
    let cartridge = Cartridge::new(vec);
    nes.load_cartridge(cartridge);

    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id("rustendo-canvas").unwrap();
    let canvas: HtmlCanvasElement = canvas
        .dyn_into::<HtmlCanvasElement>()
        .map_err(|_| ())
        .unwrap();

    let context = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<CanvasRenderingContext2d>()
        .unwrap();

    let f = Rc::new(RefCell::new(None));
    let g = Rc::clone(&f);
    let nes1 = Rc::new(RefCell::new(nes));
    let nes2 = Rc::clone(&nes1);

    let mut prev_timestamp = 0.0;
    *g.borrow_mut() = Some(Closure::wrap(Box::new(move |timestamp| {
        web_sys::console::log(&Array::of1(&JsValue::from_f64(timestamp - prev_timestamp)));
        draw(&context, &canvas, &mut nes1.borrow_mut());
        if continue_animation() {
            request_animation_frame(f.borrow().as_ref().unwrap());
        }
        prev_timestamp = timestamp;
    }) as Box<dyn FnMut(f64)>));

    nes2.borrow_mut().reset();
    request_animation_frame(g.borrow().as_ref().unwrap());
}

fn draw(context: &CanvasRenderingContext2d, canvas: &HtmlCanvasElement, nes: &mut Nes) {
    nes.clock();

    let image_data = context
        .get_image_data(0.0, 0.0, canvas.width().into(), canvas.height().into())
        .expect("failed to get ImageData");

    let mut data = image_data.data();

    for y in 0..image_data.height() {
        for x in 0..image_data.width() {
            let color = nes.color_at_coord(x, y);
            set_color_at_coord(&mut data, canvas.width(), x, y, color)
        }
    }

    let image_data = ImageData::new_with_u8_clamped_array(Clamped(&mut data), canvas.width())
        .expect("could not create image data");
    context
        .put_image_data(&image_data, 0.0, 0.0)
        .expect("could not put image data");
}

fn set_color_at_coord(
    data: &mut Clamped<Vec<u8>>,
    width: u32,
    x: u32,
    y: u32,
    color: (u8, u8, u8),
) {
    let x = x as usize;
    let y = y as usize;
    let width = width as usize;
    let red_index = y * (width * 4) + x * 4;
    data[red_index] = color.0;
    data[red_index + 1] = color.1;
    data[red_index + 2] = color.2;
    data[red_index + 3] = 0xFF;
}
