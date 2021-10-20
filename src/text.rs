use anyhow::Result;
use regex::Regex;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
pub struct TextElement {
    text: Text,
    fontset: String,
    fontsize: f64,
    align: Option<Align>,
    pos: (f64, f64),
    space: Option<(f64, f64)>,
    column: Option<usize>,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum Text {
    Multi(Vec<String>),
    Single(String),
}

impl Text {
    pub fn single(s: &str) -> Self {
        Self::Single(s.to_owned())
    }
}

#[derive(Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "lowercase")]
enum Align {
    Left,
    Center,
    Right,
}

fn filltext(text: &str, dic: &HashMap<String, Text>) -> Vec<String> {
    let mut t = vec![text.to_owned()];
    let re = Regex::new(r"\{(\w+)\}").expect("compile regex for placeholder");
    for cap in re.captures_iter(text) {
        let k = cap[1].to_owned();
        t = match dic.get(&k).unwrap_or(&Text::single("")) {
            Text::Single(s) => replace_vecstr(&k, s, &t),
            Text::Multi(ss) => ss.iter().flat_map(|s| replace_vecstr(&k, s, &t)).collect(),
        };
    }
    t
}

fn replace_vecstr(k: &str, s: &str, t: &[String]) -> Vec<String> {
    t.iter()
        .map(|u| u.replace(&format!("{{{}}}", k), s))
        .collect()
}

fn fontsize(vecstr: &[String], tl: usize, col: &Option<usize>, fontsize: f64) -> f64 {
    let len = maxlen(&vecstr);
    let mag = 1.0 / vecstr.len() as f64 * tl as f64;
    match col {
        Some(col) => fontsize * (*col as f64 / len as f64).min(mag),
        _ => fontsize * mag,
    }
}

fn maxlen(ss: &[String]) -> usize {
    ss.iter().map(|t| t.chars().count()).max().unwrap_or(1)
}

fn get_str_width(layout: &pango::Layout, s: &str, xsp: f64) -> f64 {
    layout.set_text(&s);
    let (x, _) = layout.size();
    x as f64 / pango::SCALE as f64 + xsp * s.chars().count() as f64
}

fn write_ch(cr: &cairo::Context, layout: &pango::Layout, x: f64, y: f64, ch: char) -> (f64, f64) {
    cr.move_to(x, y);
    pangocairo::update_layout(&cr, &layout);
    layout.set_text(&ch.to_string());
    let (x, y) = layout.size();
    pangocairo::show_layout(&cr, &layout);
    (
        x as f64 / pango::SCALE as f64,
        y as f64 / pango::SCALE as f64,
    )
}

fn write_str(
    cr: &cairo::Context,
    layout: &pango::Layout,
    x: f64,
    y: f64,
    xsp: f64,
    s: &str,
    align: Align,
) -> f64 {
    let w = get_str_width(layout, s, xsp);
    let mut cx = x - match align {
        Align::Left => 0.0,
        Align::Center => w / 2.0,
        Align::Right => w,
    };
    let mut cy: f64 = 0.0;
    for c in s.chars() {
        let (x, y) = write_ch(cr, layout, cx, y, c);
        cx += x + xsp;
        cy = cy.max(y);
    }
    cy
}

pub fn write(
    cr: &cairo::Context,
    layout: &pango::Layout,
    te: &TextElement,
    dic: &HashMap<String, Text>,
    descdic: &HashMap<String, String>,
) -> Result<()> {
    let (texts, tl) = match &te.text {
        Text::Multi(ss) => (ss.iter().flat_map(|s| filltext(s, dic)).collect(), ss.len()),
        Text::Single(s) => (filltext(&s, dic), 1),
    };
    let fs = fontsize(&texts, tl, &te.column, te.fontsize);
    let (xsp, ysp) = te.space.unwrap_or_default();
    let (x, mut y) = te.pos;
    let align = te.align.unwrap_or(Align::Left);
    let fontstr = descdic.get(&te.fontset).expect("font");
    let mut desc = pango::FontDescription::from_string(&fontstr);
    desc.set_size((fs * pango::SCALE as f64) as i32);
    layout.set_font_description(Some(&desc));
    for text in texts {
        let dy = write_str(cr, layout, x, y, xsp, &text, align);
        y += dy + ysp;
    }
    Ok(())
}
