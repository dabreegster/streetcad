use abstutil::CmdArgs;
use widgetry::{
    Cached, Color, Drawable, EventCtx, GfxCtx, HorizontalAlignment, Key, Line, Outcome, Panel,
    State, StyledButtons, Transition, VerticalAlignment, Widget,
};

use self::model::{Hovering, Model};

mod model;

fn main() {
    let mut args = CmdArgs::new();
    let input = args.required_free();
    args.done();
    let model = Model::load_geojson(input).unwrap();

    widgetry::run(widgetry::Settings::new("StreetCAD"), move |ctx| {
        let states = vec![Editor::new(ctx, &model)];
        (model, states)
    });
}

struct Editor {
    controls: Panel,
    draw_model: Drawable,
    hovering: Cached<Hovering, Drawable>,
    dragging: Option<(usize, usize)>,
}

impl Editor {
    fn new(ctx: &mut EventCtx, model: &Model) -> Box<dyn State<Model>> {
        let bounds = model.get_bounds();
        ctx.canvas.map_dims = (bounds.max_x, bounds.max_y);
        ctx.canvas.center_on_map_pt(bounds.center());

        Box::new(Editor {
            controls: Panel::new(Widget::col(vec![Widget::row(vec![
                Line("StreetCAD").small_heading().draw(ctx),
                ctx.style().btn_close_widget(ctx),
            ])]))
            .aligned(HorizontalAlignment::RightInset, VerticalAlignment::TopInset)
            .build(ctx),
            draw_model: ctx.upload(model.render()),
            hovering: Cached::new(),
            dragging: None,
        })
    }
}

impl State<Model> for Editor {
    fn event(&mut self, ctx: &mut EventCtx, model: &mut Model) -> Transition<Model> {
        ctx.canvas_movement();
        if ctx.redo_mouseover() {
            if let Some(pt) = ctx.canvas.get_cursor_in_map_space() {
                match self.dragging {
                    Some((idx1, idx2)) => {
                        if ctx.is_key_down(Key::LeftControl) {
                            model.move_pt((idx1, idx2), pt);
                            self.draw_model = ctx.upload(model.render());
                            self.hovering.clear();
                            self.hovering
                                .update(Some(Hovering::Point(idx1, idx2)), |key| {
                                    ctx.upload(key.render(model))
                                });
                        } else {
                            self.dragging = None;
                        }
                    }
                    None => {
                        self.hovering.update(model.compute_hovering(pt), |key| {
                            ctx.upload(key.render(model))
                        });
                        if let Some(Hovering::Point(idx1, idx2)) = self.hovering.key() {
                            if ctx.is_key_down(Key::LeftControl) {
                                self.dragging = Some((idx1, idx2));
                            }
                        }
                    }
                }
            }
        }

        match self.controls.event(ctx) {
            Outcome::Clicked(x) => match x.as_ref() {
                "close" => {
                    return Transition::Pop;
                }
                _ => unreachable!(),
            },
            _ => {}
        }

        Transition::Keep
    }

    fn draw(&self, g: &mut GfxCtx, _: &Model) {
        g.clear(Color::BLACK);

        g.redraw(&self.draw_model);
        self.controls.draw(g);
        if let Some(ref d) = self.hovering.value() {
            g.redraw(d);
        }
    }
}
