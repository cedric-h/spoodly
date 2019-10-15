use stdweb::{__js_raw_asm, js, console, js_export};

#[js_export]
fn interpret(source: String) -> String {
    use std::collections::HashMap;
    use cspeudo::{parse, Context, Evaluator, Raw, Var};

    let mut map = HashMap::new();
    map.insert(
        "DISPLAY".to_string(),
        Var::Function(Box::new(|mut args: Vec<Var>| {
            //eprintln!("args len: {}", args.len());
            let mut output = args
                .iter()
                .fold(String::new(), |acc, arg| {
                    format!("{} {}", acc, arg)
                });

            {
                let output = output.clone();
                js! { display(@{output}) };
            }

            Var::Raw(Raw::Text(output))
        })),
    );
    map.insert(
        "INPUT".to_string(),
        Var::Function(Box::new(|mut args: Vec<Var>| {
            //eprintln!("args len: {}", args.len());
            let prompt = format!("{}", args.pop().unwrap_or(Var::Raw(Raw::Text("input".to_string()))));

            let input = js! { return input(@{prompt}); }.into_string().expect("didn't give string in input");

            Var::Raw(Raw::Text(input))
        })),
    );
    map.insert(
        "+".to_string(),
        Var::Function(Box::new(|args: Vec<Var>| {
            Var::Raw(Raw::Number(
                args[0]
                    .num()
                    .and_then(|x| args[1].num().map(|y| y + x))
                    .expect("Can only add #'s!"),
            ))
        })),
    );
    let mut evalr = Evaluator::new(Context { map, parent: None });
    let output = evalr.eval(vec![parse(source)], 0);

    format!("{}", output)
}

fn main() {
    stdweb::initialize();
    stdweb::event_loop();
}
