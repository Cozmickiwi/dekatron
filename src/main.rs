use dekatron::{read_file, Dekatron};

//const TESTCODE1: [&str; 4] = ["int main()", "{", "printf(\"Hello, world!\");", "}"];

fn main() {
    let dek = Dekatron::tokenize(read_file("./cfiles/hello_world.c"));
    println!("{:#?}", dek.tokens);
}
