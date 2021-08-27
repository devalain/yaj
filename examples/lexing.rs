use yaj::*;

#[cfg(target_family = "windows")]
fn window_console_set_utf8() {
    use winapi::um::wincon::SetConsoleOutputCP;
    unsafe { SetConsoleOutputCP(65001); }
}

fn main() {
    #[cfg(target_family = "windows")]
        window_console_set_utf8();

    let json = include_str!("../big_json.txt");
    println!(
        "{:#?}",
        lex(json).iter().map(|t| t.slice).collect::<Vec<&str>>()
    );
}
