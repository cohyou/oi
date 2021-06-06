extern crate pest;

#[macro_use]
extern crate pest_derive;

#[macro_use]
extern crate conrod_core;
extern crate conrod_glium;
// #[macro_use]
extern crate conrod_winit;
extern crate find_folder;
extern crate glium;

#[derive(Parser)]
#[grammar = "_.pest"]
pub struct OiParser;

mod support;

use glium::Surface;

fn main() {
    use pest::Parser;
    // let src = "(\nwindow (canvas {width: 1024 height: 768}) \n)";
    use std::fs::File;
    use std::io::Read;
    // use std::io::{BufReader};
    let mut f = File::open("_.oi").unwrap();
    // let mut reader = BufReader::new(f);
    let mut src = String::new();
    f.read_to_string(&mut src).unwrap();
    // parseするとPairs<Rule>が返る
    let mut parse_result = OiParser::parse(Rule::oi, &src).unwrap();
    // その中はPair<Rule>
    let token_pairs = parse_result.next().unwrap();
        
    // as_str その部分のソースコード
    // as_rule どの文法規則なのか
    // as_span その部分のソースコードと位置
    // into_inner 子のPairs<Rule>
    // tokens?? Tokenはenum Start{Rule, Position}|End{Rule, Position}
    // tokenは入れ子構造をもっている入れ子の始まりでStart、終わりでEndが生成される
    // Positionは何バイト目かを持っているっぽい（行は考慮しない）
    use std::str::FromStr;
    let mut width = 0;
    let mut height = 0;
    for (i, token_pair) in token_pairs.into_inner().into_iter().enumerate() {
        match i {
            0 => {
                // println!("0{:?}", token_pair.as_str());
                width = u32::from_str(token_pair.as_str()).unwrap();
            },
            1 => {
                // println!("1{:?}", token_pair.as_str());
                height = u32::from_str(token_pair.as_str()).unwrap();
            },
            _ => unreachable!(),
        }
        // dbg!(&token_pair);
        // dbg!(&pair_def.tokens());
        // let s = pair_def.as_str();
        // dbg!(&s);
    }
    show_window(width, height);
}

fn show_window(width: u32, height: u32) {
    // let WIDTH: u32 = width;
    // let HEIGHT: u32 = height;

    // Build the window.
    let event_loop = glium::glutin::event_loop::EventLoop::new();
    let window = glium::glutin::window::WindowBuilder::new()
        .with_title("wasmiq")
        .with_inner_size(glium::glutin::dpi::LogicalSize::new(width, height));
        // .with_dimensions((WIDTH, HEIGHT).into());
    let context = glium::glutin::ContextBuilder::new()
        .with_vsync(true)
        .with_multisampling(4);
    let display = glium::backend::glutin::Display::new(window, context, &event_loop).unwrap();

    // Construct our `Ui`.
    let mut ui = conrod_core::UiBuilder::new([width as f64, height as f64]).build();

    // A unique identifier for each widget.
    let ids = Ids::new(ui.widget_id_generator());

    // Add a `Font` to the `Ui`'s `font::Map` from file.
    // let assets = find_folder::Search::KidsThenParents(3, 5)
    //     .for_folder("assets")
    //     .unwrap();
    // let font_path = assets.join("fonts/NotoSans/NotoSans-Regular.ttf");
    // let font_path = "fonts/Noto_Sans/NotoSans-Regular.ttf";
    let font_path = "fonts/NotoMono-hinted/NotoMono-Regular.ttf";
    // let font_path = "fonts/HackGen_v2.3.4/HackGen-Regular.ttf";
    ui.fonts.insert_from_file(font_path).unwrap();

    // A type used for converting `conrod_core::render::Primitives` into `Command`s that can be used
    // for drawing to the glium `Surface`.
    let mut renderer = conrod_glium::Renderer::new(&display).unwrap();

    // The image map describing each of our widget->image mappings (in our case, none).
    let image_map = conrod_core::image::Map::<glium::texture::texture2d::Texture2d>::new();

    // Some starting text to edit.
    let mut demo_text = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. \
        Mauris aliquet porttitor tellus vel euismod. Integer lobortis volutpat bibendum. Nulla \
        finibus odio nec elit condimentum, rhoncus fermentum purus lacinia. Interdum et malesuada \
        fames ac ante ipsum primis in faucibus. Cras rhoncus nisi nec dolor bibendum pellentesque. \
        Cum sociis natoque penatibus et magnis dis parturient montes, nascetur ridiculus mus. \
        Quisque commodo nibh hendrerit nunc sollicitudin sodales. Cras vitae tempus ipsum. Nam \
        magna est, efficitur suscipit dolor eu, consectetur consectetur urna."
        .to_owned();

    // Poll events from the window.
    support::run_loop(display, event_loop, move |request, display| {
        match request {
            support::Request::Event {
                event,
                should_update_ui,
                should_exit,
            } => {
                // Use the `winit` backend feature to convert the winit event to a conrod one.
                if let Some(event) = support::convert_event(&event, &display.gl_window().window()) {
                    ui.handle_event(event);
                    *should_update_ui = true;
                }

                match event {
                    glium::glutin::event::Event::WindowEvent { event, .. } => match event {
                        // Break from the loop upon `Escape`.
                        glium::glutin::event::WindowEvent::CloseRequested
                        | glium::glutin::event::WindowEvent::KeyboardInput {
                            input:
                                glium::glutin::event::KeyboardInput {
                                    virtual_keycode:
                                        Some(glium::glutin::event::VirtualKeyCode::Escape),
                                    ..
                                },
                            ..
                        } => *should_exit = true,
                        _ => {}
                    },
                    _ => {}
                }
            }
            support::Request::SetUi { needs_redraw } => {
                // Instantiate all widgets in the GUI.
                set_ui(ui.set_widgets(), &ids, &mut demo_text);

                // Get the underlying winit window and update the mouse cursor as set by conrod.
                display
                    .gl_window()
                    .window()
                    .set_cursor_icon(support::convert_mouse_cursor(ui.mouse_cursor()));

                *needs_redraw = ui.has_changed();
            }
            support::Request::Redraw => {
                // Render the `Ui` and then display it on the screen.
                let primitives = ui.draw();

                renderer.fill(display, primitives, &image_map);
                let mut target = display.draw();
                target.clear_color(0.0, 0.0, 0.0, 1.0);
                renderer.draw(display, &mut target, &image_map).unwrap();
                target.finish().unwrap();
            }
        }
    })
}

widget_ids! {
    struct Ids { canvas, text_edit, scrollbar, button, rectangle, grid, node }
}

// Declare the `WidgetId`s and instantiate the widgets.
fn set_ui(ref mut ui: conrod_core::UiCell, ids: &Ids, demo_text: &mut String) {
    use conrod_core::{color, widget, Colorable, Positionable, Sizeable, Widget};

    widget::Canvas::new()
        .scroll_kids_vertically()
        .color(color::WHITE)
        .set(ids.canvas, ui);
    
    widget::Rectangle::fill([1024.0, 64.0])
        .top_left_of(ids.canvas)
        .color(color::DARK_CHARCOAL)
        .set(ids.rectangle, ui);

    // let min_x = 0.0;
    // let max_x = std::f64::consts::PI * 2.0;
    let min_x = -1.0;
    let max_x = 1.0;
    let min_y = -1.0;
    let max_y = 1.0;

    let quarter_lines = widget::grid::Lines::step(0.5_f64).thickness(2.0);
    let sixteenth_lines = widget::grid::Lines::step(0.5_f64).thickness(1.0);
    let lines = &[
        quarter_lines.x(),
        quarter_lines.y(),
        sixteenth_lines.x(),
        sixteenth_lines.y(),
    ];

    widget::Grid::new(min_x, max_x, min_y, max_y, lines.iter().cloned())
        .color(color::rgb(0.1, 0.12, 0.15))
        .wh_of(ids.canvas)
        .middle_of(ids.canvas)
        .set(ids.grid, ui);
    
    widget::Rectangle::fill([32.0, 32.0])
        .x_y_relative(32.0 * -6.0, 32.0 * 8.0)
        // .top_left_of(ids.canvas)
        .color(color::BLUE)
        .set(ids.node, ui);

    // for edit in widget::TextEdit::new(demo_text)
    //     .color(color::WHITE)
    //     .padded_w_of(ids.canvas, 20.0)
    //     .mid_top_of(ids.canvas)
    //     // .center_justify()
    //     .line_spacing(2.5)
    //     .restrict_to_height(false) // Let the height grow infinitely and scroll.
    //     .set(ids.text_edit, ui)
    // {
    //     *demo_text = edit;
    // }

    // widget::Scrollbar::y_axis(ids.canvas)
    //     .auto_hide(true)
    //     .set(ids.scrollbar, ui);

    // use conrod_core::Labelable;
    // if widget::Button::new()
    // .w_h(40.0, 40.0)
    // .top_left_of(ui.window)
    // // .label_font_id(&ui.theme)
    // .label("1")
    // .label_font_size(16)
    // .set(ids.button, ui)
    // .was_clicked() {
    //     println!("1");
    // }
}
