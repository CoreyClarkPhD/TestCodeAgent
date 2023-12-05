use dialoguer::Select;

use crate::ai::FixCodeResult;

pub fn render_fix_code_result(result: &FixCodeResult) {
    println!("Fixed Code:\n\n");
    println!("{}", result.code);
    println!("-----------------------------------------");
    println!("Explanation:");
    println!("{}", result.explanation);
    println!("-----------------------------------------");
}

pub enum MenuOption {
    Accept,
    Tweak,
    Quit,
}

pub fn prompt_options() -> MenuOption {
    let items = vec!["Accept", "Tweak", "Quit"];

    let selection = Select::new()
        .with_prompt("What do you choose?")
        .items(&items)
        .default(0)
        .interact()
        .unwrap();

    match selection {
        0 => MenuOption::Accept,
        1 => MenuOption::Tweak,
        2 => MenuOption::Quit,
        _ => panic!("Invalid selection"),
    }
}
