use std::path::PathBuf;
use std::process::Command;
use structopt::StructOpt;

#[derive(StructOpt)]
struct CommandLineArguments {
    /// The path to the file to convert
    #[structopt(parse(from_os_str))]
    input_file: PathBuf,

    /// Output file, overwrites `webm`
    #[structopt(short = "o", parse(from_os_str))]
    output_file: Option<PathBuf>,

    /// Create a WebM instead of a gif, since its almost 2022
    #[structopt(long)]
    webm: bool,

    /// Set the width in pixels of the generated gif
    #[structopt(long, default_value = "320")]
    width: i32,

    /// Set the height in pixels of the generated gif, use '-1' to keep the aspect ratio of the input file
    #[structopt(long, default_value = "-1")]
    height: i32,
    
    /// Make it big, overwrites the `width` argument
    #[structopt(long)]
    big: bool,

    /// Don't resize, overwrites the `width`, `height` and `big` arguments
    #[structopt(long)]
    keep_size: bool,

    /// Set the framerate
    #[structopt(long, default_value = "10")]
    framerate: u32,

    /// Don't adjust the framerate, overwrites the `framerate` argument
    #[structopt(long)]
    keep_framerate: bool,
}

fn main() {
    let args = CommandLineArguments::from_args();
    let mut default_output_file = args.input_file.clone();
    default_output_file.set_extension(if args.webm { "webm" } else { "gif" });
    let output_file = args.output_file.unwrap_or(default_output_file);

    let width = if args.big { 640 } else { args.width };

    println!(
        "in {} out {}",
        args.input_file.display(),
        output_file.display()
    );

    ffmpeg_command(
        args.input_file,
        output_file,
        if args.keep_size { -1 } else { width },
        if args.keep_size { -1 } else { args.height },
        if args.keep_framerate {
            None
        } else {
            Some(args.framerate)
        },
    );
}

fn ffmpeg_command(
    input_file: PathBuf,
    output_file: PathBuf,
    width: i32,
    height: i32,
    framerate: Option<u32>,
) {
    // Filter graph definition inspired by https://superuser.com/questions/556029/how-do-i-convert-a-video-to-gif-using-ffmpeg-with-reasonable-quality/

    let fps_filter = framerate.map(|s| format!("fps={}", s));

    let scale_filter = if width == -1 && height == -1 {
        None
    } else {
        Some(format!("scale={}:{}:flags=lanczos", width, height))
    };

    let palette_filter = Some("split[s0][s1];[s0]palettegen[p];[s1][p]paletteuse".to_string());

    let filtergraph = vec![fps_filter, scale_filter, palette_filter]
        .into_iter()
        .filter(|filter| filter.is_some())
        .map(|filter| filter.unwrap())
        .collect::<Vec<String>>()
        .join(",");

    println!("Filter graph: {}", filtergraph);

    let _ = Command::new("ffmpeg")
        .arg("-i")
        .arg(&input_file)
        .arg("-vf")
        .arg(filtergraph)
        .arg(&output_file)
        .status()
        .expect("Failed to create gif");
}