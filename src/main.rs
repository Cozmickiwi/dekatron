use dekatron::Dekatron;

const TESTCODE1: [&str; 1] = ["int a = (c.b + 3);"];

fn main() {
    let dek = Dekatron::tokenize(TESTCODE1.to_vec());
    println!("{:#?}", dek.tokens);
}
