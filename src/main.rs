use widgetry::{
    Btn, Color, Drawable, EventCtx, GfxCtx, HorizontalAlignment, Line, Outcome, Panel,
    SharedAppState, State, Text, Transition, VerticalAlignment, Widget,
};

fn main() {
    abstutil::CmdArgs::new().done();

    widgetry::run(widgetry::Settings::new("StreetCAD"), |ctx| {
        (App {}, vec![Editor::new(ctx)])
    });
}

struct App {}

impl SharedAppState for App {}

struct Editor {
    controls: Panel,
    draw: Drawable,
}

impl Editor {
    fn new(ctx: &mut EventCtx) -> Box<dyn State<App>> {
        ctx.canvas.map_dims = (500.0, 500.0);

        Box::new(Editor {
            controls: Panel::new(Widget::col(vec![Widget::row(vec![
                Line("StreetCAD").small_heading().draw(ctx),
                Btn::close(ctx),
            ])]))
            .aligned(HorizontalAlignment::RightInset, VerticalAlignment::TopInset)
            .build(ctx),
            draw: ctx.upload(Text::from(Line("Yo")).render_autocropped(ctx)),
        })
    }
}

impl State<App> for Editor {
    fn event(&mut self, ctx: &mut EventCtx, _: &mut App) -> Transition<App> {
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

    fn draw(&self, g: &mut GfxCtx, _: &App) {
        g.clear(Color::BLACK);

        g.redraw(&self.draw);
        self.controls.draw(g);
    }
}
