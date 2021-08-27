use yaj::*;
fn main() {
    let json = include_str!("../big_json.txt");
    println!("{:#?}", parse(json));
}
