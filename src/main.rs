use anyhow::Result;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use structopt::StructOpt;

mod template;
mod text;

use template::CardTemplate;
use text::Text;

#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab-case")]
struct Opt {
    template: PathBuf,
    values: PathBuf,
    output: PathBuf,
    #[structopt(short = "s", long = "style", default_value = ".svgcard.css")]
    style: PathBuf,
    #[structopt(short = "p", long = "presolution", default_value = "96.0")]
    presolution: f64,
}

fn load_values(path: &Path) -> Result<HashMap<String, Text>> {
    let mut file = File::open(path)?;
    let mut buf = String::new();
    file.read_to_string(&mut buf)?;
    Ok(toml::from_str::<HashMap<String, Text>>(&buf)?)
}

fn write(
    path: &Path,
    template: &CardTemplate,
    dic: &HashMap<String, Text>,
    resolution: f64,
) -> Result<()> {
    let dim = template.dimension.clone();
    let surface = cairo::PdfSurface::new(dim.width, dim.height, path)?;
    let cr = cairo::Context::new(&surface)?;
    let pctx = pangocairo::create_context(&cr).expect("pango::Context");
    pangocairo::context_set_resolution(&pctx, resolution);
    let layout = pango::Layout::new(&pctx);

    for te in template.texts.values() {
        text::write(&cr, &layout, te, dic, &template.fontset)?;
    }

    Ok(())
}

fn main() -> Result<()> {
    let opt = Opt::from_args();
    let template = CardTemplate::from_path(&opt.template)?;
    let dic = load_values(&opt.values)?;
    write(&opt.output, &template, &dic, opt.presolution)?;
    Ok(())
}
