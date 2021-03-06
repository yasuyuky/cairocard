use crate::text;
use anyhow::Result;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

#[derive(Deserialize, Debug)]
pub struct CardTemplate {
    pub dimension: Dimension,
    pub fontset: HashMap<String, String>,
    pub fontweight: Option<HashMap<String, usize>>,
    pub imports: Option<Vec<String>>,
    pub texts: HashMap<String, text::TextElement>,
    pub svgs: Option<HashMap<String, SvgElement>>,
}

impl CardTemplate {
    pub fn from_path(path: &Path) -> Result<Self> {
        let mut file = File::open(path)?;
        let mut buf = String::new();
        file.read_to_string(&mut buf)?;
        Ok(toml::from_str::<CardTemplate>(&buf)?)
    }
}

#[derive(Clone, Deserialize, Debug)]
pub struct SvgElement {
    pub path: PathBuf,
    pub scale: f64,
    pub pos: (f64, f64),
}

#[derive(Clone, Deserialize, Debug)]
pub struct Dimension {
    pub width: f64,
    pub height: f64,
    #[serde(default = "default_offset")]
    pub offset: (isize, isize),
    #[serde(default = "default_scale")]
    pub scale: usize,
}

fn default_offset() -> (isize, isize) {
    (0, 0)
}

fn default_scale() -> usize {
    10
}
