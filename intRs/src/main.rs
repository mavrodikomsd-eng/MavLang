// MavLang mem=borrow | transpiled to Rust (safe borrows)
// main.mavl
// MavLang example v0.1
fn main() {
    let x: i32 = 5;
    let mut y = 10;
    y = y + x;
    println!("{}", y);
    println!("hello mav");
    println!("zhopa {}c1", y+y+x*95);

println!("[gui-stub] {}", "print"); // TODO real gui
}
