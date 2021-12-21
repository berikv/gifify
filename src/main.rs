
use structopt::StructOpt;
use std::path::PathBuf;
use std::process::Command;
use std::env::temp_dir;

#[derive(StructOpt)]
struct CommandLineArguments {
    /// The path to the file to convert
    #[structopt(parse(from_os_str))]
    input_file: PathBuf,

    // Output file
    #[structopt(short="o", parse(from_os_str))]
    output_file: Option<PathBuf>,
}

fn main() {
    let args = CommandLineArguments::from_args();
    let mut default_output_file = args.input_file.clone();
    default_output_file.set_extension("gif");
    let output_file = args.output_file.unwrap_or(default_output_file);

    println!("in {} out {}", args.input_file.display(), output_file.display());
    ffmpeg_command(args.input_file, output_file);
}

fn ffmpeg_command(input_file: PathBuf, output_file: PathBuf) {
    let filter = "fps=10,scale=320:-1:flags=lanczos";
    let vf = filter.to_owned() + ",palettegen";
    let lavfi = filter.to_owned() + " [x]; [x][1:v] paletteuse";

    let mut tmp_filename = input_file.to_owned();
    tmp_filename.set_extension("png");
    let tmpfile = temp_dir().with_file_name(tmp_filename.file_name().unwrap());

    println!("using tmp file {}", tmpfile.display());

    let status = Command::new("ffmpeg")
        .arg("-i").arg(&input_file)
        .arg("-vf").arg(&vf)
        .arg("-nostdin")
        .arg("-n") // don't overwrite
        .arg(&tmpfile)
        .status()
        .expect("failed to create palette");

    if !status.success() {
        panic!("Failed to create palette")
    }

    let _ = Command::new("ffmpeg")
        .arg("-i").arg(&input_file)
        .arg("-i").arg(&tmpfile)
        .arg("-lavfi").arg(&lavfi)
        .arg(&output_file)
        .status()
        .expect("failed to create gif");

    use std::fs;
    let _ = fs::remove_file(tmpfile);
}