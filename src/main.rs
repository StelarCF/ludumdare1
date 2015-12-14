extern crate piston_window;
extern crate gfx_device_gl;
extern crate find_folder;
extern crate gfx_graphics;
extern crate gfx;
extern crate nalgebra;
extern crate ncollide;
extern crate rand;
extern crate sdl2;
extern crate sdl2_mixer;
#[macro_use] extern crate conrod;

use piston_window::*;
use nalgebra::Vec1 as Vector1;
use nalgebra::Vec2 as Vector2;
use nalgebra::Rot2 as Rotate2;
use nalgebra::Pnt2 as Point2;
use nalgebra::Norm;

use rand::Rng;
use rand::StdRng;

use std::str::FromStr;

//use sdl2:;
use sdl2_mixer::{INIT_MP3, INIT_FLAC, INIT_MOD, INIT_FLUIDSYNTH, INIT_MODPLUG,
                 INIT_OGG, DEFAULT_FREQUENCY};

use sdl2_mixer::Music as Sound;

use conrod::Button as ButtonC;

use conrod::{Theme, Widget, Labelable};
use conrod::color::Color;

pub type Vec1 = Vector1<f64>;
pub type Vec2 = Vector2<f64>;
pub type Rot2 = Rotate2<f64>;
pub type Pnt2 = Point2<f64>;
pub type Ui = conrod::Ui<Glyphs>;

enum GameState {
    MainMenu,
    Game,
    Credits,
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
    fn new_rand(rng: &mut StdRng, speed_multi: f64) -> Circle {
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
        let speed = speed_multi * 100.0;
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
        if self.lifetime < 0.0 {
            self.to_delete = true;
        }
    }
}

struct Music {
    next_bass: f64,
    next_bass2: f64,
    next_bass3: f64,
    bass: Sound,
    bass2: Sound,
    bass3: Sound,
    last_combo: f64,
    combo_meter: usize,
    combo1: Sound,
    combo2: Sound,
    combo3: Sound,
    next_combo2: f64,
    next_combo3: f64,
}

impl Music {
    fn new() -> Music {
        let assets = find_folder::Search::ParentsThenKids(3, 3).for_folder("assets").unwrap();
        let bass = Sound::from_file(assets.join("sounds/music/bass.wav").as_path()).unwrap();
        let bass2 = Sound::from_file(assets.join("sounds/music/bass2.wav").as_path()).unwrap();
        let bass3 = Sound::from_file(assets.join("sounds/music/bass3.wav").as_path()).unwrap();
        let combo1 = Sound::from_file(assets.join("sounds/music/combo1.wav").as_path()).unwrap();
        let combo2 = Sound::from_file(assets.join("sounds/music/combo2.wav").as_path()).unwrap();
        let combo3 = Sound::from_file(assets.join("sounds/music/combo3.wav").as_path()).unwrap();
        Music { bass: bass, bass2: bass2, bass3: bass3,
                combo1: combo1, combo2: combo2, combo3: combo3,
                next_bass: -0.1, next_bass2: -0.1, next_bass3: -0.1,
                last_combo: 1.0, combo_meter: 0,
                next_combo2: 100000.0, next_combo3: 1000000.0}
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
    score: f64,
    on_circle: Sound,
    on_triangle: Sound,
    on_lose_triangle: Sound,
    on_death: Sound,
    music: Music,
    difficulty: f64
}

impl Game {
    fn new() -> Game {
        let assets = find_folder::Search::ParentsThenKids(3, 3).for_folder("assets").unwrap();
        let on_circle = Sound::from_file(assets.join("sounds/on_circle.wav").as_path()).unwrap();
        let on_triangle = Sound::from_file(assets.join("sounds/on_triangle.wav").as_path()).unwrap();
        let on_lose_triangle = Sound::from_file(assets.join("sounds/on_lose_triangle.wav").as_path()).unwrap();
        let on_death = Sound::from_file(assets.join("sounds/on_death.wav").as_path()).unwrap();
        Game { is_paused: false, game_state: GameState::MainMenu, time_elapsed: 0.0, grown: 0,
            colours: Vec::new(), circles: Vec::new(), triangles: Vec::new(), mx: 0.0, my: 0.0, scx: 300.0, scy: 300.0, player: Player::new(),
            next_place_triangle: 10.0, tris: 0, col_left: None, col_right: None, score: 0.0,
            on_circle: on_circle, on_triangle: on_triangle, on_death: on_death, on_lose_triangle: on_lose_triangle,
            music: Music::new(), difficulty: 0.0}
    }
    fn init(&mut self, difficulty: f64) {
        self.circles.clear();
        self.triangles.clear();
        self.score = 0.00001;
        self.next_place_triangle = 10.0;
        self.is_paused = false;
        self.time_elapsed = 0.0;
        self.grown = 0;
        self.col_left = None; self.col_right = None;
        self.tris = 0;
        self.player = Player::new();
        self.difficulty = difficulty;
        self.game_state = GameState::Game;
    }
    fn update_music(&mut self, dt: f64) {
        if self.music.next_bass2 < 0.0 {
            self.music.next_bass2 = 18.0 / (((self.grown + 10) as f64).sqrt().sqrt());
            play_sound(&self.music.bass2);
        } else {
            self.music.next_bass2 -= dt;
        }
        if self.time_elapsed > 10.0 {
            if self.music.next_bass < 0.0 {
                self.music.next_bass = 9.0 / (((self.grown + 10) as f64).sqrt().sqrt());
                play_sound(&self.music.bass);
            } else {
                self.music.next_bass -= dt;
            }
        }
        if self.time_elapsed > 20.0 {
            if self.music.next_bass3 < 0.0 {
                self.music.next_bass3 = 6.0 / (((self.grown + 10) as f64).sqrt().sqrt());
                play_sound(&self.music.bass3);
            } else {
                self.music.next_bass3 -= dt;
            }
        }
        if self.player.radius - 39.0 >= 4.0 * self.music.last_combo {
            self.music.combo_meter += 1;
            if self.music.combo_meter >= 1 {
                play_sound(&self.music.combo1);
            }
            if self.music.combo_meter >= 2 {
                self.music.next_combo2 = 0.3;
            }
            if self.music.combo_meter >= 3 {
                self.music.next_combo3 = 0.6;
            }
            self.music.last_combo = self.player.radius - 39.0;
        }
        if self.player.radius - 39.0 < self.music.last_combo * 0.8 {
            self.music.combo_meter = 0;
            self.music.last_combo = self.player.radius - 39.0;
            if self.music.last_combo < 1.0 {
                self.music.last_combo = 1.0;
            }
        }
        if self.music.next_combo2 < 0.0 {
            play_sound(&self.music.combo2);
            self.music.next_combo2 = 100000.0;
        } else {
            self.music.next_combo2 -= dt;
        }
        if self.music.next_combo3 < 0.0 {
            play_sound(&self.music.combo3);
            self.music.next_combo3 = 100000.0;
        } else {
            self.music.next_combo3 -= dt;
        }
    }
    fn on_update(&mut self, upd: UpdateArgs, ui: &mut Ui, rng: &mut StdRng) {
        match self.game_state {
            GameState::Game => {
                if !self.is_paused {
                    self.update_music(upd.dt);
                    self.time_elapsed += upd.dt;
                    self.score += self.difficulty * self.difficulty * self.time_elapsed.sqrt() * (self.player.radius - 30.0) / 10.0 * ((self.grown + 1) as f64).sqrt() / 10000.0;
                    if self.player.radius > 200.0 {
                        self.player.radius = 200.0;
                    }
                    if self.player.radius > 40.0 {
                        self.player.radius -= upd.dt * (self.player.radius - 40.0) * (self.player.radius - 40.0) / 9000.0;
                    }
                    if self.next_place_triangle < 0.0 {
                        if self.tris == 0 {
                            self.triangles.push(Triangle::new(rng, 60.0, 60.0));
                            self.tris += 1;
                        } else {
                            self.triangles.push(Triangle::new_rand(rng, self.scx * 2.0, self.scy * 2.0));
                            self.tris += 1;
                        }
                        self.next_place_triangle = 10.0 * self.difficulty;
                    } else {
                        self.next_place_triangle -= upd.dt;
                    }
                    if self.circles.len() < (self.time_elapsed.sqrt() * 3.0 + ((self.grown as f64).sqrt() * 5.0) * 0.1 * self.difficulty) as usize {
                        self.circles.push(Circle::new_rand(rng, 1.0 + (self.time_elapsed / 100.0 + (self.grown as f64) / 10.0).sqrt() * self.difficulty));
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
                                self.score += 10.0 * self.grown as f64;
                                play_sound(&self.on_circle);
                            } else {
                                if self.col_left == None && self.col_right == None {
                                    self.game_state = GameState::End;
                                    play_sound(&self.on_death);
                                } else if self.col_right == None {
                                    println!("Lost");
                                    play_sound(&self.on_lose_triangle);
                                    self.col_left = None;
                                    c.to_delete = true;
                                } else {
                                    println!("Lost");
                                    play_sound(&self.on_lose_triangle);
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
                                play_sound(&self.on_triangle);
                            } else if self.col_right == None {
                                self.col_right = Some(t.colour);
                                t.to_delete = true;
                                play_sound(&self.on_triangle);
                            }
                        }
                    }
                    self.triangles = self.triangles.iter().cloned().filter(|x| x.to_delete == false).collect();
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
                        TEXT4,
                        TEXT5
                    }

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

                    let point1 = "Point your mouse where you want to move\nEat circles of your color to grow!";
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

                    let mut score = String::from_str("Score: ").unwrap();
                    let a = (self.score).to_string();
                    let (a, b) = a.split_at(a.find('.').unwrap_or(a.len()));
                    score = score + a + "." + &b[1..2];

                    Text::new(&score[..])
                    .color(color::white())
                    .mid_top()
                    .align_text_left()
                    .line_spacing(10.0)
                    .set(TEXT5, ui);
                });
            }
            GameState::MainMenu => {
                ui.set_widgets(|ui|{
                    use conrod::{color, Colorable, Positionable, Text};

                    // Generate a unique const `WidgetId` for each widget.
                    widget_ids!{
                        TEXT,
                        BUTTON,
                    }

                    let text = "Carket\n\nChoose difficulty (press key on keyboard):\n1 - Easy\n2 - Medium\n3 - Hard\n4 - UltraHD\nC - Credits";

                    Text::new(&text)
                    .color(color::white())
                    .middle()
                    .align_text_left()
                    .line_spacing(10.0)
                    .set(TEXT, ui);

                });
            }
            GameState::Credits => {
                ui.set_widgets(|ui|{
                    use conrod::{color, Colorable, Positionable, Text};

                    // Generate a unique const `WidgetId` for each widget.
                    widget_ids!{
                        TEXT,
                        BUTTON,
                    }

                    let text = "Carket\n\nCoded by: StelarCF\n\nSpecial Thanks To:\nWindowsBunny from irc.mozilla.org #rust_gamedev for building my game for Windows\nMysticPing for some useful ideas\nCziken20 for giving me a couple of seconds of thinking my stream wasn't empty\n\nPress Enter/Return to go back";

                    Text::new(&text)
                    .color(color::white())
                    .middle()
                    .align_text_left()
                    .line_spacing(10.0)
                    .set(TEXT, ui);

                });
            }
            _ => {
                ui.set_widgets(|ui|{
                    use conrod::{color, Colorable, Positionable, Text};

                    // Generate a unique const `WidgetId` for each widget.
                    widget_ids!{
                        MASTER,
                        TOP,
                        MID,
                        BOTTOM,
                        TEXT,
                    }

                    let mut score = String::from_str("You died\n Your score was: ").unwrap();
                    let a = (self.score).to_string();
                    let (a, b) = a.split_at(a.find('.').unwrap_or(a.len()));
                    score = score + a + "." + &b[1..2] + "\n\nPress Enter/Return to go back to the main menu";

                    Text::new(&score[..])
                    .color(color::white())
                    .middle()
                    .align_text_middle()
                    .line_spacing(20.0)
                    .set(TEXT, ui);
                });
            }
        }
    }
    fn on_draw(&mut self, ren: RenderArgs, e: PistonWindow, ui: &mut Ui) {
        self.scx = (ren.width / 2) as f64;
        self.scy = (ren.height / 2) as f64;
        match self.game_state {
            GameState::MainMenu => {
                self.draw_main_menu(ren, e, ui);
            }
            GameState::Game => {
                self.draw_game(ren, e, ui);
            }
            GameState::End => {
                self.draw_end(ren, e, ui);
            }
            GameState::Credits => {
                self.draw_credits(ren, e, ui);
            }
        }
    }
    fn on_input(&mut self, inp: Input) {
        match self.game_state {
            GameState::Game => {
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
                                    Key::Escape => {
                                        self.game_state = GameState::MainMenu;
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
            GameState::MainMenu => {
                match inp {
                    Input::Release(k) => {
                        match k {
                            Button::Keyboard(k) => {
                                match k {
                                    Key::D1 => {
                                        self.init(0.3);
                                    }
                                    Key::D2 => {
                                        self.init(0.6);
                                    }
                                    Key::D3 => {
                                        self.init(1.0);
                                    }
                                    Key::D4 => {
                                        self.init(2.0);
                                    }
                                    Key::D0 => {
                                        self.init(5.0);
                                    }
                                    Key::C => {
                                        self.game_state = GameState::Credits;
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
            GameState::Credits => {
                match inp {
                    Input::Release(k) => {
                        match k {
                            Button::Keyboard(k) => {
                                match k {
                                    Key::Return => {
                                        self.game_state = GameState::MainMenu;
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
            _ => {
                match inp {
                    Input::Release(k) => {
                        match k {
                            Button::Keyboard(k) => {
                                match k {
                                    Key::Return => {
                                        self.game_state = GameState::MainMenu;
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

    fn draw_end(&mut self, ren: RenderArgs, e: PistonWindow, ui: &mut Ui) {
        e.draw_2d(|c, g| {
            clear([0.2, 0.1, 0.1, 1.0], g);
            ui.draw(c, g);
        });
    }

    fn draw_main_menu(&mut self, ren: RenderArgs, e: PistonWindow, ui: &mut Ui) {
        e.draw_2d(|c, g| {
            clear([0.1, 0.1, 0.1, 1.0], g);
            ui.draw(c, g);
        });
    }

    fn draw_credits(&mut self, ren: RenderArgs, e: PistonWindow, ui: &mut Ui) {
        e.draw_2d(|c, g| {
            clear([0.1, 0.1, 0.1, 1.0], g);
            ui.draw(c, g);
        });
    }
}

fn main() {
    {
        let window: PistonWindow = WindowSettings::new(
            "Carket",
            [1080, 1080]
        )
        .samples(4)
        .exit_on_esc(false)
        .build()
        .unwrap();
        let assets = find_folder::Search::ParentsThenKids(3, 3).for_folder("assets").unwrap();
        let font_path = assets.join("fonts/Raleway-Regular.ttf");
        let theme = Theme::default();
        let glyph_cache = Glyphs::new(&font_path, window.factory.borrow().clone());
        let mut ui = Ui::new(glyph_cache.unwrap(), theme);

        let sdl = sdl2::init().unwrap();
        let _audio = sdl.audio().unwrap();

        let _ = sdl2_mixer::open_audio(DEFAULT_FREQUENCY, 0x8010u16, 2, 1024);
        sdl2_mixer::allocate_channels(32);

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
    sdl2_mixer::quit();
}

fn play_sound(music: &Sound) {
    music.play(1).unwrap();
}
