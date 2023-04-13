use schemajen::*;

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    let bin_name = env!("CARGO_BIN_NAME");

    let support_str = ACCUMULATOR_SUPPORT_LIST
        .iter()
        .fold(String::new(), |total, s| total + "\t" + s + "\n");

    if args.iter().any(|s| s == "-h" || s == "--help") {
        eprintln!(
            "{bin_name}, v{}

Learn more at https://github.com/davnotdev/schemajen.

Auto-magically convert JSON into language bindings.
Run with `{bin_name} [accumulator] [typename] [file]`.
View this very message with `{bin_name} --help` or `{bin_name} -h`.

This version was compiled with the following accumulators:

{}",
            env!("CARGO_PKG_VERSION"),
            support_str
        );
        return;
    }

    let Some(accumulator) = args.get(1) else {
        eprintln!("Expected language accumulator, see `{bin_name} -h`");
        return;
    };

    let Some(typename) = args.get(2) else {
        eprintln!("Expected type name, see `{bin_name} -h`");
        return;
    };

    let Some(filename) = args.get(3) else {
        eprintln!("Expected file, see `{bin_name} -h`");
        return;
    };

    let file = std::fs::read_to_string(filename);
    if let Err(e) = file {
        eprintln!("Failed to open file with: {}", e);
        return;
    };
    let file = file.unwrap();

    let Some(mut accumulator) = accumulator_choose_with_str(accumulator) else {
            eprintln!("That accumulator does not exist");
            return;
    };
    let res = generate(accumulator.as_mut(), typename, &file);

    if let Err(e) = &res {
        eprintln!("Codegen failed with: {:?}", e);
        return;
    }

    println!("{}", res.unwrap());
}
