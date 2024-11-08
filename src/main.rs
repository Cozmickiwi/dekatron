use std::time::Instant;

use dekatron::{read_file, Dekatron, TokenType};

//const TESTCODE1: [&str; 4] = ["int main()", "{", "printf(\"Hello, world!\");", "}"];

fn main() {
    //let dek = Dekatron::tokenize(read_file("./cfiles/hello_world.c"));
    let now = Instant::now();
    let dek = Dekatron::tokenize(read_file("./cfiles/test3.c"));
    let el = now.elapsed();
    println!("{:?}", el);
    //println!("{:#?}", dek.tokens);
    // print vals for testing
    let mut indent_level = 0;
    for t in dek.tokens {
        if t[0].raw.as_str() == "}" {
            indent_level -= 1;
        }
        for _ in 0..indent_level {
            print!("  ");
        }
        if t[0].raw.as_str() == "{" {
            indent_level += 1;
        }
        for r in t {
            print!("{} ", r.raw);
        }

        println!();
    }
}
