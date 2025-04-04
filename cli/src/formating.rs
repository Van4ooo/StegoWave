use colored::Colorize;

pub fn print_success_helper(left_msg: &str) {
    eprintln!("{} {}", "[SUCCESS]".green().bold(), left_msg.white().bold());
}

#[macro_export]
macro_rules! print_success {
    (file: $out_path:expr) => {
        print_success_helper("The output file is created");
        println!("{}", $out_path.to_string().red().bold().underline());
        eprintln!();
    };

    (message: $message:expr) => {
        print_success_helper("Secret message received");
        println!("{}", $message.red().bold().underline());
        eprintln!();
    };
}
