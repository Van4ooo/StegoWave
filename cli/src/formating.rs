use colored::Colorize;
use std::fmt::Debug;

#[inline]
pub fn print_error(error: String) {
    eprintln!(
        "{} {}",
        "[ERROR]".red().bold(),
        error.white().italic().underline()
    );
}

#[inline]
pub fn print_info(msg: String) {
    println!("{} {}", "[INFO]".cyan().bold(), msg.white().italic());
}

#[inline]
pub fn print_format(left_msg: &str, right_msg: &str) {
    println!(
        "{} {} -> {}",
        "[FORMAT]".blue().bold(),
        left_msg.white().italic(),
        right_msg.white().italic()
    );
}

#[inline]
pub fn print_success<L, R>(left_msg: L, right_msg: R)
where
    L: Debug,
    R: Debug,
{
    println!(
        "{} {} -> {}",
        "[SUCCESS]".green().bold(),
        format!("{:?}", left_msg).yellow().bold(),
        format!("{:?}", right_msg).red().bold().underline()
    )
}
