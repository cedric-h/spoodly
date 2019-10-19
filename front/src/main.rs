use stdweb::{__js_raw_asm, js, js_export};

#[js_export]
// wraps spoodly::interpret and provides the web STD.
fn interpret(src: String) -> String {
    use spoodly::{Context, Raw, eval::Var};

    let mut webstd = Context::std();
    webstd.map.insert(
        "DISPLAY".to_string(),
        Var::Function(Box::new(|args: Vec<Var>| {
            //eprintln!("args len: {}", args.len());
            let output = args.iter().fold(String::new(), |acc, arg| {
                format!("{} {}", acc, arg).trim().to_owned()
            });

            {
                let output = output.clone();
                js! { display(@{output}) };
            }

            Var::Raw(Raw::Text(output))
        })),
    );
    webstd.map.insert(
        "INPUT".to_string(),
        Var::Function(Box::new(|mut args: Vec<Var>| {
            //eprintln!("args len: {}", args.len());
            let prompt = format!(
                "{}",
                args.pop()
                    .unwrap_or(Var::Raw(Raw::Text("input".to_string())))
            );

            let input = js! { return input(@{prompt}); }
                .into_string()
                .expect("didn't give string in input");

            Var::Raw(Raw::Text(input))
        })),
    );

    match spoodly::interpret(src, webstd) {
        // nobody wants to see the normal program output for the time being.
        Ok(_) => String::new(),
        Err(msg) => msg,
    }
}

fn main() {
    stdweb::initialize();
    stdweb::event_loop();
}
