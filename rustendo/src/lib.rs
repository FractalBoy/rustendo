use js_sys::Uint8Array;
use rustendo_lib::cartridge::Cartridge;
use rustendo_lib::nes::Nes;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::{Clamped, JsCast};
use web_sys::{
    CanvasRenderingContext2d, Event, HtmlCanvasElement, ImageData, KeyboardEvent, Window,
};

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

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .unwrap();
}

fn add_event_listener<T>(event: &str, f: &Closure<dyn FnMut(T)>)
where
    T: AsRef<Event>,
{
    window()
        .add_event_listener_with_callback(event, f.as_ref().unchecked_ref())
        .unwrap();
}

fn get_viewport_size() -> (i32, i32) {
    let document_element = window().document().unwrap().document_element().unwrap();
    (
        document_element.client_width(),
        document_element.client_height(),
    )
}

#[wasm_bindgen(start)]
pub fn startup() {
    utils::set_panic_hook();
    setup_canvas();
}

fn setup_canvas() {
    let canvas = get_canvas();

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

fn get_canvas() -> HtmlCanvasElement {
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id("rustendo-canvas").unwrap();
    canvas
        .dyn_into::<HtmlCanvasElement>()
        .map_err(|_| ())
        .unwrap()
}

fn get_canvas_rendering_context(canvas: &HtmlCanvasElement) -> CanvasRenderingContext2d {
    canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<CanvasRenderingContext2d>()
        .unwrap()
}

fn create_canvas_and_rendering_context() -> (HtmlCanvasElement, CanvasRenderingContext2d) {
    let canvas = window()
        .document()
        .unwrap()
        .create_element(&"canvas")
        .unwrap()
        .dyn_into::<HtmlCanvasElement>()
        .map_err(|_| ())
        .unwrap();

    canvas.set_width(NES_WIDTH);
    canvas.set_height(NES_HEIGHT);

    let context = get_canvas_rendering_context(&canvas);

    (canvas, context)
}

#[wasm_bindgen]
pub fn render(byte_array: Uint8Array) {
    let nes = load_cartridge(byte_array);
    let nes = Rc::new(RefCell::new(nes));

    setup_keydown_handler(&nes);
    setup_keyup_handler(&nes);
    setup_animation(&nes);
}

fn load_cartridge(byte_array: Uint8Array) -> Nes {
    let mut nes = Nes::new();

    let vec = byte_array.to_vec();
    let cartridge = Cartridge::new(vec);
    nes.load_cartridge(cartridge);

    nes
}

fn setup_animation(nes: &Rc<RefCell<Nes>>) {
    let canvas = get_canvas();
    let context = get_canvas_rendering_context(&canvas);
    let (renderer, renderer_context) = create_canvas_and_rendering_context();
    let moved_nes = Rc::clone(nes);
    let nes = Rc::clone(&moved_nes);

    let mut screen = [0; (NES_WIDTH * NES_HEIGHT * 4) as usize];

    let moved_closure = Rc::new(RefCell::new(None));
    let closure = Rc::clone(&moved_closure);

    *closure.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        request_animation_frame(moved_closure.borrow().as_ref().unwrap());

        while !moved_nes.borrow_mut().clock() {}

        draw(
            &mut screen,
            &context,
            &canvas,
            &renderer_context,
            &renderer,
            &moved_nes.borrow(),
        );
    }) as Box<dyn FnMut()>));

    nes.borrow_mut().reset();
    request_animation_frame(closure.borrow().as_ref().unwrap());
}

fn setup_keydown_handler(nes: &Rc<RefCell<Nes>>) {
    let nes = Rc::clone(nes);

    let keydown_handler = Closure::wrap(Box::new(move |event: web_sys::KeyboardEvent| {
        let mut nes = nes.borrow_mut();
        let controller = nes.controller();

        match event.key().as_str() {
            "a" | "A" => controller.press_a(),
            "s" | "S" => controller.press_b(),
            "ArrowLeft" => controller.press_left(),
            "ArrowRight" => controller.press_right(),
            "ArrowUp" => controller.press_up(),
            "ArrowDown" => controller.press_down(),
            "x" | "X" => controller.press_start(),
            "z" | "Z" => controller.press_select(),
            _ => return,
        };
    }) as Box<dyn FnMut(KeyboardEvent)>);

    add_event_listener::<KeyboardEvent>("keydown", &keydown_handler);
    keydown_handler.forget();
}

fn setup_keyup_handler(nes: &Rc<RefCell<Nes>>) {
    let nes = Rc::clone(nes);

    let keyup_handler = Closure::wrap(Box::new(move |event: web_sys::KeyboardEvent| {
        let mut nes = nes.borrow_mut();
        let controller = nes.controller();

        match event.key().as_str() {
            "a" | "A" => controller.lift_a(),
            "s" | "S" => controller.lift_b(),
            "ArrowLeft" => controller.lift_left(),
            "ArrowRight" => controller.lift_right(),
            "ArrowUp" => controller.lift_up(),
            "ArrowDown" => controller.lift_down(),
            "x" | "X" => controller.lift_start(),
            "z" | "Z" => controller.lift_select(),
            _ => return,
        };
    }) as Box<dyn FnMut(KeyboardEvent)>);

    add_event_listener::<KeyboardEvent>("keyup", &keyup_handler);
    keyup_handler.forget();
}

fn draw(
    data: &mut [u8],
    context: &CanvasRenderingContext2d,
    canvas: &HtmlCanvasElement,
    renderer_context: &CanvasRenderingContext2d,
    renderer: &HtmlCanvasElement,
    nes: &Nes,
) {
    let screen = nes.get_screen();

    for y in 0..NES_HEIGHT {
        for x in 0..NES_WIDTH {
            set_color_at_coord(data, x, y, screen[y as usize][x as usize])
        }
    }

    let image_data = ImageData::new_with_u8_clamped_array(Clamped(data), NES_WIDTH)
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

fn set_color_at_coord(data: &mut [u8], x: u32, y: u32, color: (u8, u8, u8)) {
    let x = x as usize;
    let y = y as usize;
    let width = NES_WIDTH as usize;
    let red_index = y * (width * 4) + x * 4;

    data[red_index] = color.0;
    data[red_index + 1] = color.1;
    data[red_index + 2] = color.2;
    data[red_index + 3] = 0xFF;
}
