use colored::Colorize;
use std::fmt::Debug;

#[inline]
pub fn print_format(output_msg: &str) {
    eprintln!(
        "{} {} -> {}",
        "[FORMAT]".blue().bold(),
        "input file".white().italic(),
        output_msg.white().italic()
    );
}

pub fn print_input_path<L: Debug>(left_msg: L) {
    eprint!(
        "{} {} -> ",
        "[SUCCESS]".green().bold(),
        format!("{:?}", left_msg).yellow().bold()
    );
}

#[macro_export]
macro_rules! print_success {
    (file: $out_path:expr, $inp_path:expr) => {
        print_format("output file");
        print_input_path($inp_path);
        println!("{}", format!("{:?}", $out_path).red().bold().underline());
    };

    (message: $message:expr, $inp_path:expr) => {
        print_format("secret message");
        print_input_path($inp_path);
        println!("{}", format!("{}", $message).red().bold().underline());
    };
}
