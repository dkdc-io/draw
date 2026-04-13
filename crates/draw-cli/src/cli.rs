use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

use draw_core::document::Document;

#[derive(Parser, Debug)]
#[command(name = "draw")]
#[command(about = "Local-first drawing tool")]
#[command(version)]
pub struct Args {
    /// Open the desktop app
    #[cfg(feature = "app")]
    #[arg(short = 'a', long)]
    pub app: bool,

    /// Open the webapp
    #[cfg(feature = "webapp")]
    #[arg(short = 'w', long)]
    pub webapp: bool,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(clap::Subcommand, Debug)]
pub enum Command {
    /// Create a new drawing
    New {
        /// Drawing name
        #[arg(default_value = "untitled")]
        name: String,
    },
    /// Open an existing drawing
    Open {
        /// Path to .draw.json file
        file: PathBuf,
    },
    /// List saved drawings
    List,
    /// Export drawing to SVG
    ExportSvg {
        /// Path to .draw.json file
        file: PathBuf,
        /// Output SVG path (defaults to same name with .svg extension)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Export drawing to PNG
    ExportPng {
        /// Path to .draw.json file
        file: PathBuf,
        /// Output PNG path (defaults to same name with .png extension)
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Scale factor (default: 2x for retina)
        #[arg(short, long, default_value_t = 2.0)]
        scale: f32,
    },
}

/// Parse `argv` as the draw CLI and dispatch the matching subcommand.
///
/// Clap handles `--help` / `--version` / argument errors itself by calling
/// `process::exit`, so those paths never return here.
///
/// # Errors
/// Returns an error if a subcommand fails: I/O errors loading or exporting a
/// drawing, a requested drawing id not found, or the embedded webapp/app
/// failing to launch.
pub fn run_cli(argv: impl IntoIterator<Item = impl Into<String>>) -> Result<()> {
    let argv: Vec<String> = argv.into_iter().map(Into::into).collect();

    // No args = help
    if argv.len() <= 1 {
        Args::parse_from(["draw", "--help"]);
    }

    let args = Args::parse_from(&argv);

    #[cfg(feature = "app")]
    if args.app {
        return draw_app::run_app(None);
    }

    #[cfg(feature = "webapp")]
    if args.webapp {
        return draw_webapp::run_webapp(None);
    }

    match args.command {
        Some(Command::New { name }) => {
            let doc = Document::new(name);
            let path = draw_core::storage::save_to_storage(&doc)?;
            println!("Created: {} ({})", doc.name, path.display());

            #[cfg(feature = "webapp")]
            return draw_webapp::run_webapp(Some(doc.id));

            #[cfg(not(feature = "webapp"))]
            {
                println!("Run with --webapp to open in browser");
                Ok(())
            }
        }
        Some(Command::Open { file }) => {
            let doc = draw_core::storage::load(&file)?;
            println!("Loaded: {} ({} elements)", doc.name, doc.elements.len());

            #[cfg(feature = "webapp")]
            return draw_webapp::run_webapp(Some(doc.id));

            #[cfg(not(feature = "webapp"))]
            {
                println!("Run with --webapp to open in browser");
                Ok(())
            }
        }
        Some(Command::List) => {
            let drawings = draw_core::storage::list_drawings()?;
            if drawings.is_empty() {
                println!("No saved drawings.");
            } else {
                for (name, path) in drawings {
                    println!("  {} ({})", name, path.display());
                }
            }
            Ok(())
        }
        Some(Command::ExportSvg { file, output }) => {
            let doc = draw_core::storage::load(&file)?;
            let svg = draw_core::export_svg(&doc);
            let out_path = output.unwrap_or_else(|| file.with_extension("svg"));
            std::fs::write(&out_path, &svg)?;
            println!("Exported SVG to {}", out_path.display());
            Ok(())
        }
        Some(Command::ExportPng {
            file,
            output,
            scale,
        }) => {
            let doc = draw_core::storage::load(&file)?;
            let png = draw_core::export_png_with_scale(&doc, scale)?;
            let out_path = output.unwrap_or_else(|| file.with_extension("png"));
            std::fs::write(&out_path, &png)?;
            println!("Exported PNG to {}", out_path.display());
            Ok(())
        }
        None => {
            // No subcommand but also no flags = help
            Args::parse_from(["draw", "--help"]);
            unreachable!()
        }
    }
}
