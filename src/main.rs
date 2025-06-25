mod calc;

fn main() {
    println!("Calculator");
    println!("Type in 'q' to quit.");
    loop {
        println!("Please enter an expr below");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();
        if input == "q" {
            println!("Goodbye.");
            break;
        } else if input.is_empty() {
            continue;
        } else {
            match calc::expr(&input) {
                Ok(res) => println!("Result: {res}"),
                Err(err_lint) => println!("{err_lint}")
            }
        }
    }
}
