use rustendo_lib::cartridge::Cartridge;
use rustendo_lib::nes::Nes;
use std::cell::RefCell;
use std::f64;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::{Clamped, JsCast};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, ImageData, Window};

// Leaving this import here to make it easier to use the macro when debugging.
#[allow(unused_imports)]
use rustendo_lib::log;

mod utils;

const NES_WIDTH: u32 = 256;
const NES_HEIGHT: u32 = 240;

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
        .expect("could not request animation frame");
}

fn add_event_listener<T>(event: &str, f: &Closure<dyn FnMut(T)>)
where
    T: AsRef<web_sys::Event>,
{
    window()
        .add_event_listener_with_callback(event, f.as_ref().unchecked_ref())
        .expect("could not create event listener");
}

fn get_viewport_size() -> (i32, i32) {
    let document_element = window()
        .document()
        .expect("no document exists")
        .document_element()
        .unwrap();
    (
        document_element.client_width(),
        document_element.client_height(),
    )
}

#[wasm_bindgen(start)]
pub fn startup() {
    utils::set_panic_hook();

    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id("rustendo-canvas").unwrap();
    let canvas: HtmlCanvasElement = canvas
        .dyn_into::<HtmlCanvasElement>()
        .map_err(|_| ())
        .unwrap();

    let (viewport_width, viewport_height) = get_viewport_size();
    let multiples_of_width = viewport_width as u32 / NES_WIDTH;
    let multiples_of_height = viewport_height as u32 / NES_HEIGHT;
    let smallest_multiple = if multiples_of_width < multiples_of_height {
        multiples_of_width
    } else {
        multiples_of_height
    };

    let smallest_multiple = if smallest_multiple == 0 {
        1
    } else {
        smallest_multiple
    };

    canvas.set_width(NES_WIDTH * smallest_multiple);
    canvas.set_height(NES_HEIGHT * smallest_multiple);
}

#[wasm_bindgen]
pub fn render(byte_array: js_sys::Uint8Array) {
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
    let nes3 = Rc::clone(&nes1);

    let mut prev_timestamp = 0.0;
    *g.borrow_mut() = Some(Closure::wrap(Box::new(move |timestamp| {
        request_animation_frame(f.borrow().as_ref().unwrap());

        // Only draw once every 1/60th of a second.
        if timestamp - prev_timestamp >= 1000.0 / 60.0 {
            while !nes1.borrow_mut().clock() {}
            draw(&context, &canvas, &nes1.borrow());
            prev_timestamp = timestamp;
        }
    }) as Box<dyn FnMut(f64)>));

    nes2.borrow_mut().reset();

    let keydown_handler =
        Closure::wrap(Box::new(
            move |event: web_sys::KeyboardEvent| match event.key().as_str() {
                "a" | "A" => nes2.borrow().controller().borrow_mut().press_a(),
                "s" | "S" => nes2.borrow().controller().borrow_mut().press_b(),
                "ArrowLeft" => nes2.borrow().controller().borrow_mut().press_left(),
                "ArrowRight" => nes2.borrow().controller().borrow_mut().press_right(),
                "ArrowUp" => nes2.borrow().controller().borrow_mut().press_up(),
                "ArrowDown" => nes2.borrow().controller().borrow_mut().press_down(),
                "x" | "X" => nes2.borrow().controller().borrow_mut().press_start(),
                "z" | "Z" => nes2.borrow().controller().borrow_mut().press_select(),
                _ => return,
            },
        ) as Box<dyn FnMut(web_sys::KeyboardEvent)>);

    let keyup_handler =
        Closure::wrap(Box::new(
            move |event: web_sys::KeyboardEvent| match event.key().as_str() {
                "a" | "A" => nes3.borrow().controller().borrow_mut().lift_a(),
                "s" | "S" => nes3.borrow().controller().borrow_mut().lift_b(),
                "ArrowLeft" => nes3.borrow().controller().borrow_mut().lift_left(),
                "ArrowRight" => nes3.borrow().controller().borrow_mut().lift_right(),
                "ArrowUp" => nes3.borrow().controller().borrow_mut().lift_up(),
                "ArrowDown" => nes3.borrow().controller().borrow_mut().lift_down(),
                "x" | "X" => nes3.borrow().controller().borrow_mut().lift_start(),
                "z" | "Z" => nes3.borrow().controller().borrow_mut().lift_select(),
                _ => return,
            },
        ) as Box<dyn FnMut(web_sys::KeyboardEvent)>);

    add_event_listener::<web_sys::KeyboardEvent>("keydown", &keydown_handler);
    add_event_listener::<web_sys::KeyboardEvent>("keyup", &keyup_handler);

    keydown_handler.forget();
    keyup_handler.forget();

    request_animation_frame(g.borrow().as_ref().unwrap());
}

fn draw(context: &CanvasRenderingContext2d, canvas: &HtmlCanvasElement, nes: &Nes) {
    let renderer = window()
        .document()
        .unwrap()
        .create_element(&"canvas")
        .expect("could not create canvas")
        .dyn_into::<HtmlCanvasElement>()
        .map_err(|_| ())
        .expect("");

    renderer.set_width(NES_WIDTH);
    renderer.set_height(NES_HEIGHT);

    let renderer_context = renderer
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<CanvasRenderingContext2d>()
        .unwrap();

    let mut data = vec![0; (NES_WIDTH * NES_HEIGHT * 4) as usize];

    for y in 0..NES_HEIGHT {
        for x in 0..NES_WIDTH {
            let color = nes.color_at_coord(x, y);
            set_color_at_coord(&mut data, NES_WIDTH, x, y, color)
        }
    }

    let image_data = ImageData::new_with_u8_clamped_array(Clamped(&mut data), NES_WIDTH)
        .expect("could not create image data");

    renderer_context
        .put_image_data(&image_data, 0.0, 0.0)
        .expect("could not put image data");

    context
        .draw_image_with_html_canvas_element_and_dw_and_dh(
            &renderer,
            0.0,
            0.0,
            canvas.width().into(),
            canvas.height().into(),
        )
        .expect("could not draw canvas onto context");
}

fn set_color_at_coord(data: &mut Vec<u8>, width: u32, x: u32, y: u32, color: (u8, u8, u8)) {
    let x = x as usize;
    let y = y as usize;
    let width = width as usize;
    let red_index = y * (width * 4) + x * 4;
    data[red_index] = color.0;
    data[red_index + 1] = color.1;
    data[red_index + 2] = color.2;
    data[red_index + 3] = 0xFF;
}
