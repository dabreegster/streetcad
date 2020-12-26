use abstutil::CmdArgs;
use widgetry::{
    Btn, Color, Drawable, EventCtx, GfxCtx, HorizontalAlignment, Line, Outcome, Panel, State,
    Transition, VerticalAlignment, Widget,
};

use self::model::Model;

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
    draw: Drawable,
}

impl Editor {
    fn new(ctx: &mut EventCtx, model: &Model) -> Box<dyn State<Model>> {
        let bounds = model.get_bounds();
        ctx.canvas.map_dims = (bounds.max_x, bounds.max_y);

        Box::new(Editor {
            controls: Panel::new(Widget::col(vec![Widget::row(vec![
                Line("StreetCAD").small_heading().draw(ctx),
                Btn::close(ctx),
            ])]))
            .aligned(HorizontalAlignment::RightInset, VerticalAlignment::TopInset)
            .build(ctx),
            draw: ctx.upload(model.render()),
        })
    }
}

impl State<Model> for Editor {
    fn event(&mut self, ctx: &mut EventCtx, _: &mut Model) -> Transition<Model> {
        ctx.canvas_movement();

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

        g.redraw(&self.draw);
        self.controls.draw(g);
    }
}
