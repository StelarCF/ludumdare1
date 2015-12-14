extern crate piston_window;
extern crate gfx_device_gl;
extern crate find_folder;
extern crate gfx_graphics;
extern crate gfx;
extern crate nalgebra;
extern crate ncollide;
extern crate rand;
extern crate rodio;
#[macro_use] extern crate conrod;

use piston_window::*;
use nalgebra::Vec1 as Vector1;
use nalgebra::Vec2 as Vector2;
use nalgebra::Rot2 as Rotate2;
use nalgebra::Pnt2 as Point2;
use nalgebra::Norm;

use rand::Rng;
use rand::StdRng;

use rodio::Source;
use std::time::Duration;

use std::str::FromStr;

use std::io::BufReader;

use conrod::{Theme, Widget};
use conrod::color::Color;

pub type Vec1 = Vector1<f64>;
pub type Vec2 = Vector2<f64>;
pub type Rot2 = Rotate2<f64>;
pub type Pnt2 = Point2<f64>;
pub type Ui = conrod::Ui<Glyphs>;

enum GameState {
    MainMenu,
    Game,
    End
}

struct Player {
    x: f64, y: f64,
    radius: f64,
    colour_state: usize
}

impl Player {
    fn new() -> Player {
        Player { x: 0.0, y: 0.0, radius: 40.0, colour_state: 0 }
    }
}

#[derive(Clone)]
struct Circle {
    x: f64, y: f64,
    vx: f64, vy: f64,
    radius: f64,
    outside_colour: usize,
    inside_colour: usize,
    to_delete: bool
}

impl Circle {
    fn new() -> Circle {
        Circle {x: 0.0, y: 0.0, vx: 0.0, vy: 0.0, radius: 40.0, outside_colour: 0, inside_colour: 1, to_delete: false}
    }
    fn new2(x: f64, y: f64) -> Circle {
        Circle {x: x, y: y, vx: 0.0, vy: 0.0, radius: 40.0, outside_colour: 0, inside_colour: 1, to_delete: false}
    }
    fn new_rand(rng: &mut StdRng) -> Circle {
        let (mut x, mut y) = (0.0, 0.0);
        let (mut vx, mut vy) = (0.0, 0.0);
        if rng.gen() { // top or bottom
            if rng.gen() { // top
                y = -960.0;
                x = rng.gen::<f64>() * 1920.0 - 960.0;
            } else {
                y = 960.0;
                x = rng.gen::<f64>() * 1920.0 - 960.0;
            }
        } else { // left or right
            if rng.gen() { // left
                x = -960.0;
                y = rng.gen::<f64>() * 1920.0 - 960.0;
            } else {
                x = 960.0;
                y = rng.gen::<f64>() * 1920.0 - 960.0;
            }
        }
        let n = Vec2::new(x, y).norm();
        vy = -y / n;
        vx = -x / n;
        let speed = 100.0;
        vy *= speed * (rng.gen::<f64>() + 0.5);
        vx *= speed * (rng.gen::<f64>() + 0.5);
        let color_in: usize = rng.gen();
        let color_in = color_in % 6;
        let color_out: usize = rng.gen();
        let color_out = color_out % 6;
        Circle {
            x: x, y: y,
            vx: vx, vy: vy,
            radius: 40.0,
            outside_colour: color_in,
            inside_colour: color_out,
            to_delete: false
        }
    }
    fn update(&mut self, dt: f64) {
        self.x += self.vx * dt;
        self.y += self.vy * dt;
    }
}

#[derive(Clone)]
struct Triangle {
    x: f64, y: f64,
    radius: f64,
    colour: usize,
    lifetime: f64,
    to_delete: bool
}

impl Triangle {
    fn new(rng: &mut StdRng, px: f64, py: f64) -> Triangle {
        Triangle {
            x: px,
            y: py,
            lifetime: 30.0,
            radius: 20.0,
            to_delete: false,
            colour: rng.gen::<usize>() % 6
        }
    }
    fn new_rand(rng: &mut StdRng, bx: f64, by: f64) -> Triangle {
        let x: f64 = rng.gen::<f64>() * bx - bx / 2.0;
        let y: f64 = rng.gen::<f64>() * by - by / 2.0;
        Triangle {
            x: x,
            y: y,
            lifetime: 30.0,
            radius: 20.0,
            to_delete: false,
            colour: rng.gen::<usize>() % 6
        }
    }
    fn update(&mut self, dt: f64) {
        self.lifetime -= dt;
        if(self.lifetime < 0.0) {
            self.to_delete = true;
        }
    }
}

struct Game {
    is_paused: bool,
    game_state: GameState,
    time_elapsed: f64,
    grown: i64,
    pub colours: Vec<Color>,
    scx: f64, scy: f64,
    mx: f64, my: f64,
    player: Player,
    circles: Vec<Circle>,
    triangles: Vec<Triangle>,
    next_place_triangle: f64,
    tris: i64,
    col_left: Option<usize>,
    col_right: Option<usize>,
    score: f64
}

impl Game {
    fn new() -> Game {
        Game { is_paused: false, game_state: GameState::Game, time_elapsed: 0.0, grown: 0, colours: Vec::new(), circles: Vec::new(), triangles: Vec::new(), mx: 0.0, my: 0.0, scx: 300.0, scy: 300.0, player: Player::new(),
        next_place_triangle: 10.0, tris: 0, col_left: None, col_right: None, score: 0.0}
    }
    fn init(&mut self) {
        self.circles.clear();
        self.triangles.clear();
    }
    fn on_update(&mut self, upd: UpdateArgs, ui: &mut Ui, rng: &mut StdRng) {
        match self.game_state {
            GameState::Game => {
                if !self.is_paused {
                    self.time_elapsed += upd.dt;
                    if self.player.radius > 200.0 {
                        self.player.radius = 200.0;
                    }
                    if self.player.radius > 40.0 {
                        println!("{}", self.player.radius);
                        self.player.radius -= upd.dt * (self.player.radius - 40.0) * (self.player.radius - 40.0) / 1000.0;
                        println!("{}", self.player.radius);
                    }
                    if self.next_place_triangle < 0.0 {
                        if self.tris == 0 {
                            self.triangles.push(Triangle::new(rng, 60.0, 60.0));
                            self.tris += 1;
                        } else {
                            self.triangles.push(Triangle::new_rand(rng, self.scx * 2.0, self.scy * 2.0));
                            self.tris += 1;
                        }
                        self.next_place_triangle = 10.0;
                    } else {
                        self.next_place_triangle -= upd.dt;
                    }
                    if rng.gen::<f64>() < 0.015 + self.time_elapsed.sqrt() / 10000.0 + (self.grown as f64) / 8000.0 && self.circles.len() < (self.time_elapsed.sqrt() * 3.0 + (self.grown as f64).sqrt() * 5.0) as usize {
                        self.circles.push(Circle::new_rand(rng));
                    }
                    let p = Vec2::new(self.player.x, self.player.y);
                    let f = Vec2::new(self.mx - self.scx, self.my - self.scy);
                    let delta = f - p;
                    let speed = 500.0;
                    if delta.norm() < 5.0 {
                        self.player.x = f.x;
                        self.player.y = f.y;
                    } else {
                        let delta = delta.normalize();
                        self.player.x += upd.dt * speed * delta.x;
                        self.player.y += upd.dt * speed * delta.y;
                    }
                    for ref mut c in &mut self.circles {
                        c.update(upd.dt);
                        let p = Vec2::new(self.player.x, self.player.y);
                        let f = Vec2::new(c.x, c.y);
                        let delta = f - p;
                        if delta.norm() * 2.0 < self.player.radius + c.radius {
                            if c.outside_colour == self.player.colour_state {
                                self.grown += 1;
                                self.player.radius += 2.0;
                                self.player.colour_state = c.inside_colour;
                                c.to_delete = true;
                            } else {
                                if self.col_left == None && self.col_right == None {
                                    self.game_state = GameState::End;
                                } else if self.col_right == None {
                                    println!("Removing left");
                                    self.col_left = None;
                                    c.to_delete = true;
                                } else {
                                    println!("Removing right");
                                    self.col_right = None;
                                    c.to_delete = true;
                                }
                            }
                        } else {
                            if c.x > 1200.0 || c.x < -1200.0 || c.y > 1200.0 || c.y < -1200.0 {
                                c.to_delete = true;
                            }
                        }
                    }
                    self.circles = self.circles.iter().cloned().filter(|x| x.to_delete == false).collect();
                    for ref mut t in &mut self.triangles {
                        t.update(upd.dt);
                        let p = Vec2::new(self.player.x, self.player.y);
                        let f = Vec2::new(t.x, t.y);
                        let delta = f - p;
                        if delta.norm() * 2.0 < self.player.radius + t.radius {
                            if self.col_left == None {
                                self.col_left = Some(t.colour);
                                t.to_delete = true;
                                println!("Got left");
                            } else if self.col_right == None {
                                self.col_right = Some(t.colour);
                                t.to_delete = true;
                                println!("Got right");
                            }
                        }
                    }
                    self.triangles = self.triangles.iter().cloned().filter(|x| x.to_delete == false).collect();
                }
            }
            _ => {}
        }
        ui.set_widgets(|ui|{
            use conrod::{color, Colorable, Positionable, Text};

            // Generate a unique const `WidgetId` for each widget.
            widget_ids!{
                MASTER,
                TOP,
                MID,
                BOTTOM,
                TEXT,
                TEXT2,
                TEXT3,
                TEXT4
            }

            // Our `Canvas` tree, upon which we will place our text widgets.

            let mut time = String::from_str("Time Alive: ").unwrap();
            let a = (self.time_elapsed).to_string();
            let (a, b) = a.split_at(a.find('.').unwrap_or(a.len()));
            time = time + a + "." + &b[1..2];

            Text::new(&time[..])
            .color(color::white())
            .top_left()
            .align_text_left()
            .line_spacing(10.0)
            .set(TEXT, ui);

            let mut grown = String::from_str("Grown: ").unwrap();
            grown = grown + &(self.grown).to_string()[..];

            Text::new(&grown[..])
            .color(color::white())
            .top_right()
            .align_text_left()
            .line_spacing(10.0)
            .set(TEXT2, ui);

            let point1 = "Point your mouse where you want to move";
            let point2 = "Collect triangles to gain powerups\nYou can use them to change your color instantly by pressing the mouse buttons!\nThey can also act as shields";
            if self.time_elapsed < 10.0 {
                Text::new(&point1)
                .color(color::white())
                .middle()
                .align_text_left()
                .line_spacing(20.0)
                .set(TEXT3, ui);
            } else if self.time_elapsed < 20.0 {
                Text::new(&point2)
                .color(color::white())
                .middle()
                .align_text_middle()
                .line_spacing(10.0)
                .set(TEXT3, ui);
            }

            let mut nro = String::from_str("Number of objects: ").unwrap();
            nro = nro + &(self.circles.len()).to_string()[..];
            Text::new(&nro[..])
            .color(color::white())
            .bottom_right()
            .align_text_left()
            .line_spacing(10.0)
            .set(TEXT4, ui);
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
            GameState::End => {
                self.draw_end(ren, e);
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
            Input::Release(k) => {
                match k {
                    Button::Keyboard(k) => {
                        match k {
                            Key::P => {
                                self.is_paused = !(self.is_paused);
                            }
                            _ => {}
                        }
                    }
                    Button::Mouse(m) => {
                        match m {
                            MouseButton::Left => {
                                let mut tmp: usize = 0;
                                match self.col_left {
                                    Some(col_left) => {
                                        tmp = self.player.colour_state;
                                        self.player.colour_state = col_left;
                                    }
                                    _ => {}
                                }
                                if self.col_left != None {
                                    self.col_left = Some(tmp);
                                }
                            }
                            MouseButton::Right => {
                                let mut tmp: usize = 0;
                                match self.col_right {
                                    Some(col_right) => {
                                        tmp = self.player.colour_state;
                                        self.player.colour_state = col_right;
                                    }
                                    _ => {}
                                }
                                if self.col_right != None {
                                    self.col_right = Some(tmp);
                                }
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn draw_game(&mut self, ren: RenderArgs, e: PistonWindow, ui: &mut Ui) {
        e.draw_2d(|c, g| {
            let conrod::color::Rgba(rr, gg, bb, aa) = conrod::color::hsl(((((self.time_elapsed * self.time_elapsed / 10.0) as i64) % 628) as f32) / 100.0, 0.8, 0.1).to_rgb();
            clear([rr, gg, bb, aa], g);
            let rekt = [-self.player.radius / 2.0, -self.player.radius / 2.0, self.player.radius, self.player.radius];
            let conrod::color::Rgba(rr, gg, bb, aa) = self.colours[self.player.colour_state].to_rgb();
            ellipse([rr, gg, bb, aa], rekt, c.transform.trans((ren.width / 2) as f64, (ren.height / 2) as f64).trans(self.player.x, self.player.y), g);
            for ref mut cc in &self.circles {
                let conrod::color::Rgba(rr, gg, bb, aa) = self.colours[cc.outside_colour].to_rgb();
                let rekt = [-cc.radius / 2.0, -cc.radius / 2.0, cc.radius, cc.radius];
                ellipse([rr, gg, bb, aa], rekt, c.transform.trans((ren.width / 2) as f64, (ren.height / 2) as f64).trans(cc.x, cc.y), g);
                let rekt = [-cc.radius / 4.0, -cc.radius / 4.0, cc.radius / 2.0, cc.radius / 2.0];
                let conrod::color::Rgba(rr, gg, bb, aa) = self.colours[cc.inside_colour].to_rgb();
                ellipse([rr, gg, bb, aa], rekt, c.transform.trans((ren.width / 2) as f64, (ren.height / 2) as f64).trans(cc.x, cc.y), g);
            }
            for ref mut tt in &self.triangles {
                let conrod::color::Rgba(rr, gg, bb, aa) = self.colours[tt.colour].to_rgb();
                polygon([rr, gg, bb, aa], &[[0.0, -tt.radius / 2.0], [tt.radius / 2.0, tt.radius / 2.0], [-tt.radius / 2.0, tt.radius / 2.0]], c.transform.trans((ren.width / 2) as f64, (ren.height / 2) as f64).trans(tt.x, tt.y), g);
            }
            if self.col_left != None {
                let conrod::color::Rgba(rr, gg, bb, aa) = self.colours[self.col_left.unwrap()].to_rgb();
                polygon([rr, gg, bb, aa], &[[0.0, -60.0 / 2.0], [60.0 / 2.0, 60.0 / 2.0], [-60.0 / 2.0, 60.0 / 2.0]], c.transform.trans(50.0, 50.0), g);
            }
            if self.col_right != None {
                let conrod::color::Rgba(rr, gg, bb, aa) = self.colours[self.col_right.unwrap()].to_rgb();
                polygon([rr, gg, bb, aa], &[[0.0, -60.0 / 2.0], [60.0 / 2.0, 60.0 / 2.0], [-60.0 / 2.0, 60.0 / 2.0]], c.transform.trans(ren.width as f64 - 50.0, 50.0), g);
            }
            ui.draw(c, g);
        });
    }

    fn draw_end(&mut self, ren: RenderArgs, e: PistonWindow) {
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
        [1080, 1080]
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
    let colours = vec![(0.0, 0.5, 0.55, 1.0), (1.04, 0.5, 0.55, 1.0),
      (2.08, 0.5, 0.55, 1.0), (3.14, 0.5, 0.55, 1.0),
      (4.18, 0.5, 0.55, 1.0), (5.22, 0.5, 0.55, 1.0)]; // in total 6 colours
    game.colours = colours.into_iter().map(|(h, s, l, _)| { let (r, g, b) = conrod::color::hsl_to_rgb(h, s, l); conrod::color::rgb(r, g, b)}).collect();
    let mut rng = StdRng::new().unwrap();
    for e in window.ups(60) {
        ui.handle_event(&e);
        match e.event {
            Some(Event::Update(upd)) => {
                game.on_update(upd, &mut ui, &mut rng);
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
