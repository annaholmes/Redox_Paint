// pulled from https://github.com/redox-os/orbtk/blob/master/src/widgets/image.rs
extern crate orbtk;
extern crate orbclient;
extern crate orbimage;

use std::rc::Rc;
use orbtk::{Color, Window, Image, Button, Enter, Rect, List, Text, TextBox, Point, Renderer, Event, Label};

use orbtk::traits::{Click, Place, EventFilter};
use orbtk::widgets::Widget;
use std::thread;

use std::cell::{Cell, RefCell, RefMut};
use orbtk::cell::{CloneCell, CheckSet};
use std::path::Path;
use std::sync::{Arc, Mutex};

#[derive(PartialEq, Clone, Debug)]
pub enum Tool{
    Pen,
    Erase,
    Fill,
    Line,
    Rectangle,
    Circle,
}

fn get_name(t : Tool) -> String{
    let s = match t{
        Tool::Pen => {"Pen(p)"},
        Tool::Erase => {"Erase(e)"}
        Tool::Rectangle => {"Rectangle(r)"}
        Tool::Line => {"Line(l)"}
        _   => {"other"}
    };
    return String::from(s);
}

pub struct PaintCanvas {
    pub rect: Cell<Rect>,
    pub image: RefCell<orbimage::Image>,
    click_callback: RefCell<Option<Arc<Fn(&PaintCanvas, Point)>>>,
    click_pos: Rc<RefCell<Option<Point>>>,
    pub tool : Arc<Mutex<Tool>>,
    pub size : Arc<Mutex<i32>>,
    pub color : Arc<Mutex<orbtk::Color>>,
}

impl PaintCanvas {
    pub fn new(width: u32, height: u32, tool : Arc<Mutex<Tool>>,
        color : Arc<Mutex<orbtk::Color>>, size : Arc<Mutex<i32>>) -> Arc<Self> {

        Self::from_image(orbimage::Image::new(width, height), tool, color, size)
    }

    pub fn from_color(width: u32, height: u32, color: Color, tool : Arc<Mutex<Tool>>,
        penColor : Arc<Mutex<orbtk::Color>>, size : Arc<Mutex<i32>>) -> Arc<Self> {
        Self::from_image(orbimage::Image::from_color(width, height, color), tool, penColor, size)
    }


    pub fn from_image(image: orbimage::Image, arcTool : Arc<Mutex<Tool>>,
        c : Arc<Mutex<orbtk::Color>>, s : Arc<Mutex<i32>>) -> Arc<Self> {
        Arc::new(PaintCanvas {
            rect: Cell::new(Rect::new(0, 0, image.width(), image.height())),
            image: RefCell::new(image),
            click_callback: RefCell::new(None),
            click_pos: Rc::new(RefCell::new(None)),
            tool : arcTool,
            size: s,
            color: c,
        })
    }

    pub fn handlePen(&self, mut prev_opt : std::cell::RefMut<Option<Point>>, point : Point, color : orbtk::Color){
        if let Some(prev_position) = *prev_opt {
            let mut image = self.image.borrow_mut();

            let size;
            {
                size = *self.size.lock().unwrap();
            }

            for i in 0..size{
                for j in 0..size{
                    let m : i32 = i - (size/2);
                    let n : i32 = j - (size/2);
                    image.line(prev_position.x+m,
                           prev_position.y+n,
                           point.x+m,
                           point.y+n,
                           color);
                }
            }
        }
        *prev_opt = Some(point);
    }

    fn DrawHorizontal(&self, size:i32, x1:i32, x2:i32, yCoord:i32){
        let c_0 = self.color.clone();
        let mut c = c_0.lock().unwrap();
        let mut image : RefMut<orbimage::Image> = self.image.borrow_mut();
        for k in 0..size{
            image.line(x1,
                       yCoord+k,
                       x2,
                       yCoord+k,
                       *c);
        }
        for k in 0..(-size){
            image.line(x1,
                       yCoord-k,
                       x2,
                       yCoord-k,
                       *c);
        }
    }

    fn DrawVertical(&self, size:i32, y1:i32, y2:i32, xCoord:i32){
        let c_0 = self.color.clone();
        let mut c = c_0.lock().unwrap();
        let mut image : RefMut<orbimage::Image> = self.image.borrow_mut();
        for k in 0..size{
            image.line(xCoord+k,
                       y1,
                       xCoord+k,
                       y2,
                       *c);
        }
        for k in 0..(-size){
            image.line(xCoord-k,
                       y1,
                       xCoord-k,
                       y2,
                       *c);
        }
    }

    // TODO fix stupid corner bug
        // size goes out too far

    pub fn handleRect(&self, mut prev_opt : std::cell::RefMut<Option<Point>>, point : Point){
        if let Some(prev_position) = *prev_opt {

            let size;
            {
                size = *self.size.lock().unwrap();
            }

            self.DrawHorizontal(size, prev_position.x, point.x, prev_position.y);
            self.DrawHorizontal(-size, point.x, prev_position.x, point.y);
            self.DrawVertical(size, prev_position.y, point.y, prev_position.x);
            self.DrawVertical(-size, point.y, prev_position.y, point.x);
        }
        *prev_opt = None;
    }

    pub fn handleLine(&self, mut prev_opt : std::cell::RefMut<Option<Point>>, point : Point){
        if let Some(prev_position) = *prev_opt {
            let size;
            {
                size = *self.size.lock().unwrap();
            }
            let mut image : RefMut<orbimage::Image> = self.image.borrow_mut();
            let c_0 = self.color.clone();
            let mut c = c_0.lock().unwrap();
            for i in 0..size{
                for j in 0..size{
                    let m : i32 = i - (size/2);
                    let n : i32 = j - (size/2);
                    image.line(prev_position.x+m,
                           prev_position.y+n,
                           point.x+m,
                           point.y+n,
                           *c);
                }
            }
            *prev_opt = None;
        }
    }

}

// fn inRect(size:i32, xBounds:i32, yBounds:i32, x:i32, y:i32) -> bool {
//     return is_lat(size, xBounds, x) ||
//            is_lat(size, yBounds, y);
// }
fn is_lat(size:i32, leftBound:i32, rightBound:i32, c:i32) -> bool {
    return c <= leftBound + (size/2) ||
           c >= rightBound - 1 - (size/2);
}

impl Click for PaintCanvas {
    fn emit_click(&self, point: Point) {
        if let Some(ref click_callback) = *self.click_callback.borrow() {
            click_callback(self, point);
        }
    }

    fn on_click<T: Fn(&Self, Point) + 'static>(&self, func: T) -> &Self {
        *self.click_callback.borrow_mut() = Some(Arc::new(func));
        self
    }
}

impl Place for PaintCanvas {}

impl Widget for PaintCanvas {
    fn rect(&self) -> &Cell<Rect> {
        &self.rect
    }

    fn draw(&self, renderer: &mut Renderer, _focused: bool) {
        let rect = self.rect.get();
        let image = self.image.borrow();
        renderer.image(rect.x, rect.y, image.width(), image.height(), image.data());
    }

    fn event(&self, event: Event, focused: bool, redraw: &mut bool) -> bool {

        match event {

            Event::Mouse { point, left_button, .. } => {
                let t_0 = self.tool.clone();
                let mut t = t_0.lock().unwrap();
                let click = self.click_pos.clone();
                let mut prev_opt = click.borrow_mut();
                if left_button {
                    if *t == Tool::Pen || *t == Tool::Erase {
                        let c_0 = self.color.clone();
                        let c = c_0.lock().unwrap();
                        let color = if *t == Tool::Erase {orbtk::Color::rgb(255,255,255)} else {*c};
                        self.handlePen(prev_opt, point, color);
                    } else  if *t == Tool::Rectangle || *t == Tool::Line{
                        match *prev_opt{
                            None    => {*prev_opt = Some(point);}
                            _       => {}
                        }
                    }
                }
                else{
                    if *t == Tool::Rectangle {
                        self.handleRect(prev_opt, point);
                    }
                    else if *t == Tool::Line {
                        self.handleLine(prev_opt, point);
                    }
                    else{
                        *prev_opt = None;
                    }
                }
            }
            Event::Text { c } => {
                let t_0 = self.tool.clone();
                let mut t = t_0.lock().unwrap();
                if c == 'p' {*t = Tool::Pen; print!("Changing to pen tool");}
                else if c == 'e' {*t = Tool::Erase; print!("Changing to erase tool");}
                else if c == 'f' {*t = Tool::Fill; print!("Changing to fill tool");}
                else if c == 'l' {*t = Tool::Line; print!("Changing to line tool");}
                else if c == 'r' {*t = Tool::Rectangle; print!("Changing to rectangle tool");}
                else if c == '=' || c == '+' {
                    let mut s = *self.size.lock().unwrap();
                    s += 1;
                    print!("Pen larger");
                }
                else if c == '-' {
                    let mut s = *self.size.lock().unwrap();
                    s = if s > 1 {s-1} else{1};
                    print!("Pen smaller");
                }
                else{}//print!("Unknown tool {} ", c);}

            }
            _ => {}//print!("Something else!");}
        }

        true
    }
}

fn get_u8(textbox : &TextBox) -> Option<u8>{
    match textbox.text.get().parse::<usize>(){
        Ok(u) => {
            let mut ret = u as u8;
            if u > 255{
                ret = 255 as u8;
                textbox.text("255");
            }
            let ret = if u > 255 {255 as u8} else {u as u8};
            Some(ret)
        }
        Err(_) => {
            textbox.text("0");
            None
        }
    }
}

pub fn main() {
    let tool = Arc::new(Mutex::new(Tool::Pen));

    let color = Arc::new(Mutex::new(orbtk::Color::rgb(0, 0, 0)));
    let size = Arc::new(Mutex::new(7 as i32));

    let c = color.clone();
    let t = tool.clone();
    let s = size.clone();
    thread::spawn(move || {
        // as you add tools, add them to the tool list 2 make the button
        let toolList = vec![Tool::Pen, Tool::Erase, Tool::Rectangle, Tool::Line];
        let mut tools = Window::new(Rect::new(5, 30, 105, 420), "Tools");
        //let menu = Image::from_color(25, 420, Color::rgb(255, 255, 255));
        //menu.position(15, 15);
        //tools.add(&menu);
        let mut count = 0;

        /*let colorDisplay = Arc::new(Mutex::new(
            Image::from_color(50,50,orbtk::Color::rgb(0,0,0)).position(0,10)));
        {
            tools.add(*colorDisplay);
        }*/

        {
            let mut r_text = Label::new();
            r_text.position(10,10).size(40,30).text("r:");
            tools.add(&r_text);
        }
        {
            let c_r = c.clone();
            let mut r = TextBox::new();
            r.position(50, 10).size(60, 30).text("0").on_enter(move |r: &TextBox| {
                let c_1 = c_r.clone();
                let mut c_2 = c_1.lock().unwrap();
                match get_u8(r){
                    Some(u) => {
                        *c_2 = orbtk::Color::rgb(u, c_2.g(), c_2.b());
                    }
                    None => {}
                }
            });
            tools.add(&r);
        }

        {
            let mut g_text = Label::new();
            g_text.position(10,40).size(40,30).text("g:");
            tools.add(&g_text);
        }
        {
            let c_g = c.clone();
            let g = TextBox::new();
            g.position(50, 40).size(60, 30).text("0").on_enter(move |g: &TextBox| {
                match get_u8(g){
                    Some(u) => {
                        let c_1 = c_g.clone();
                        let mut c_2 = c_1.lock().unwrap();
                        *c_2 = orbtk::Color::rgb(c_2.r(), u, c_2.b());
                    }
                    None => {}
                }
            });
            tools.add(&g);
        }

        {
            let mut b_text = Label::new();
            b_text.position(10,70).size(40,30).text("b:");
            tools.add(&b_text);
        }
        {
            let c_b = c.clone();
            let b = TextBox::new();
            b.position(50, 70).size(60, 30).text("0").on_enter(move |b: &TextBox| {
                match get_u8(b){
                    Some(u) => {
                        let c_1 = c_b.clone();
                        let mut c_2 = c_1.lock().unwrap();
                        *c_2 = orbtk::Color::rgb(c_2.r(), c_2.g(), u);
                    }
                    None => {}
                }
            });
            tools.add(&b);
        }

        {
            let mut s_text = Label::new();
            s_text.position(10,100).size(40,30).text("size:");
            tools.add(&s_text);
        }
        {
            let ss = s.clone();
            let size_box = TextBox::new();
            size_box.position(50, 100).size(60,30).text("7").on_enter(move |size_box: &TextBox| {
                match size_box.text.get().parse::<i32>(){
                    Ok(s) => {
                        let mut ss2 = ss.lock().unwrap();
                        *ss2 = s;
                    }
                    Err(e) => {}
                }
            });
            tools.add(&size_box);
        }

        for i in toolList {

            let mut button = Button::new();

            let i_temp = i.clone();
            button.text(get_name(i_temp));


            button.position(10, 130 + (30 * count) as i32).size(100, 30);
            let tTemp1 = t.clone();
            button.on_click(move |button : &Button, point : Point| {
                let t0 = tTemp1.clone();
                let mut t1 = t0.lock().unwrap();
                let toolCopy = i.clone();
                *t1 = toolCopy;
            });


            tools.add(&button);
            count += 1;
        }



        tools.exec();
    });
    let (display_width, display_height) = orbclient::get_display_size().unwrap();
    let width = display_width - 115;
    let height = display_height - 70;

    let mut window = Window::new(Rect::new(110, 30, width, height), "Canvas");


    let t2 = tool.clone();
    let c2 = color.clone();
    let s2 = size.clone();
    let canvas = PaintCanvas::from_color(width, height, Color::rgb(255, 255, 255), t2, c2, s2);
    canvas.position(0, 0);
    window.add(&canvas);

    let whatever = Arc::new(Mutex::new(Tool::Pen));
    let bs = Arc::new(Mutex::new(orbtk::Color::rgb(0,0,0)));
    let bs2 = Arc::new(Mutex::new(0 as i32));
    let dumbFix = PaintCanvas::from_color(0, 0, Color::rgb(255, 255, 255), whatever, bs, bs2);
    window.add(&dumbFix);

    window.exec();

}
