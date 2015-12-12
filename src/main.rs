extern crate piston_window;
extern crate gfx_device_gl;
extern crate find_folder;
extern crate gfx_graphics;
extern crate gfx;
extern crate nalgebra;
extern crate ncollide;
#[macro_use] extern crate conrod;

use piston_window::*;
use gfx_device_gl::{Resources, Output, CommandBuffer};
use gfx_graphics::GfxGraphics;
use std::f64::consts::PI;
use nalgebra::Vec1 as Vector1;
use nalgebra::Vec2 as Vector2;
use nalgebra::Rot2 as Rotate2;
use nalgebra::Pnt2 as Point2;
use nalgebra::Norm;
use std::str::FromStr;

use conrod::{Theme, Widget};

pub type Vec1 = Vector1<f64>;
pub type Vec2 = Vector2<f64>;
pub type Rot2 = Rotate2<f64>;
pub type Pnt2 = Point2<f64>;
pub type Ui = conrod::Ui<Glyphs>;

/*
[10:33:20] Victor (MysticPing): I'm thinking you have a circle in the midle
[10:33:21] Victor (MysticPing): it has a color
[10:33:26] Victor (MysticPing): you move left and right
[10:33:35] Victor (MysticPing): other small circles fly down, some your color some not
[10:33:41] Victor (MysticPing): you need to get hit by the ones that are your color
[10:33:45] Victor (MysticPing): if you do you grow a bit and change color
*/

enum GameState {
    MainMenu,
    Game,
    Won
}

struct Player {
    x: f64, y: f64,
    colour_state: usize
}

impl Player {
    fn new() -> Player {
        Player { x: 0.0, y: 0.0, colour_state: 0 }
    }
}

struct Game {
    game_state: GameState,
    time_elapsed: f64,
    pub colours: Vec<conrod::color::Color>,
    scx: f64, scy: f64,
    mx: f64, my: f64,
    player: Player
}

impl Game {
    fn new() -> Game {
        Game { game_state: GameState::Game, time_elapsed: 0.0, colours: Vec::new(), mx: 0.0, my: 0.0, scx: 300.0, scy: 300.0, player: Player::new()}
    }
    fn on_update(&mut self, upd: UpdateArgs, ui: &mut Ui) {
        self.time_elapsed += upd.dt;
        match self.game_state {
            GameState::Game => {
                let p = Vec2::new(self.player.x, self.player.y);
                let f = Vec2::new(self.mx - self.scx, self.my - self.scy);
                let delta = f - p;
                let speed = 40.0;
                if(delta.norm() < 0.4) {
                    self.player.x = f.x;
                    self.player.y = f.y;
                } else {
                    let delta = delta.normalize();
                    self.player.x += upd.dt * speed * delta.x;
                    self.player.y += upd.dt * speed * delta.y;
                }
            }
            _ => {}
        }
        ui.set_widgets(|ui|{
            use conrod::{color, Colorable, Positionable, Sizeable, Split, Text};

            // Generate a unique const `WidgetId` for each widget.
            widget_ids!{
                MASTER,
                TOP,
                MID,
                BOTTOM,
                TEXT,
                TEXT2,
                TEXT3
            }

            // Our `Canvas` tree, upon which we will place our text widgets.

            let mut score = String::from_str("Elapsed Time ").unwrap();
            let a = (self.time_elapsed).to_string();
            let (a, b) = a.split_at(a.find('.').unwrap_or(a.len()));
            score = score + a + "." + &b[1..2];

            Text::new(&score[..])
               .color(color::white())
               .top_left()
               .align_text_left()
               .line_spacing(10.0)
               .set(TEXT, ui);
       });
   }
   fn on_draw(&mut self, ren: RenderArgs, e: PistonWindow, ui: &mut Ui) {
       self.scx = (ren.width / 2) as f64;
       self.scy = (ren.height / 2) as f64;
       match self.game_state {
           GameState::MainMenu => {
               self.draw_main_menu(ren, e);
           }
           GameState::Game => {
               self.draw_game(ren, e, ui);
           }
           GameState::Won => {
               self.draw_won(ren, e);
           }
       }
   }
   fn on_input(&mut self, inp: Input) {
       match inp {
           Input::Move(mot) => {
            match mot {
                Motion::MouseCursor(x, y) => {
                    self.mx = x;
                    self.my = y;
                }
                _ => {}
            }
        }
           _ => {}
       }
   }

   fn draw_game(&mut self, ren: RenderArgs, e: PistonWindow, ui: &mut Ui) {
       e.draw_2d(|c, g| {
           let conrod::color::Rgba(rr, gg, bb, aa) = conrod::color::hsl(((((self.time_elapsed * 100.0) as i64) % 628) as f32) / 100.0, 0.8, 0.1).to_rgb();
           clear([rr, gg, bb, aa], g);
           let rekt = [-20.0, -20.0, 40.0, 40.0];
           for i in 0..8 {
               let conrod::color::Hsla(rr, gg, bb, aa) = self.colours[self.player.colour_state].to_hsl();
           }
           let conrod::color::Rgba(rr, gg, bb, aa) = self.colours[self.player.colour_state].to_rgb();
           ellipse([rr, gg, bb, aa], rekt, c.transform.trans((ren.width / 2) as f64, (ren.height / 2) as f64).trans(self.player.x, self.player.y), g);
           ui.draw(c, g);
       });
   }

   fn draw_won(&mut self, ren: RenderArgs, e: PistonWindow) {
       e.draw_2d(|c, g| {
           clear([0.6, 0.6, 0.8, 1.0], g);
       });
   }

   fn draw_main_menu(&mut self, ren: RenderArgs, e: PistonWindow) {
       e.draw_2d(|c, g| {
           clear([0.8, 0.8, 0.8, 1.0], g);
       });
   }
}

fn main() {
   let window: PistonWindow = WindowSettings::new(
       "piston-tutorial",
       [1080, 720]
   )
   .samples(4)
   .exit_on_esc(true)
   .build()
   .unwrap();
   let assets = find_folder::Search::ParentsThenKids(3, 3).for_folder("assets").unwrap();
   let font_path = assets.join("fonts/Raleway-Regular.ttf");
   let theme = Theme::default();
   let glyph_cache = Glyphs::new(&font_path, window.factory.borrow().clone());
   let mut ui = Ui::new(glyph_cache.unwrap(), theme);
   let mut game = Game::new();
   let colours = vec![(0.0, 0.8, 0.6, 1.0), (1.04, 0.8, 0.6, 1.0),
                      (2.08, 0.8, 0.6, 1.0), (3.14, 0.8, 0.6, 1.0),
                      (4.18, 0.8, 0.6, 1.0), (5.22, 0.8, 0.6, 1.0)]; // in total 6 colours
   game.colours = colours.into_iter().map(|(h, s, l, _)| { println!("{} {} {}", h, s, l); let (r, g, b) = conrod::color::hsl_to_rgb(h, s, l); println!("{} {} {}", r, g, b); conrod::color::rgb(r, g, b)}).collect();
   for e in window.ups(60) {
       ui.handle_event(&e);
       match e.event {
           Some(Event::Update(upd)) => {
               game.on_update(upd, &mut ui);
           }
           Some(Event::Render(ren)) => {
               game.on_draw(ren, e, &mut ui);
           }
           Some(Event::Input(inp)) => {
               game.on_input(inp);
           }
           _ => {}
       }
   }
}
