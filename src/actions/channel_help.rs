const HELP_MESSAGE_FOR_CHANNEL: &'static str = "
**To get a colour:**

1) Pick the colour you want on the list.
2) Type the name of the colour and send it in this channel.
3) When you see the green tick react, you will have the role.
";

fn make_usage_example(name: &String) -> String {
    format!(
        "To get the colour named {0}, type:

    {0}
        ",
        name
    )
}

pub fn generate_help_message(names: Vec<String>) -> String {
    let usage_examples = names
        .iter()
        .take(2)
        .map(make_usage_example)
        .collect::<Vec<String>>();

    format!(
        " 
{}

*Example usage:*

{}

*Colours avaliable:*
",
        HELP_MESSAGE_FOR_CHANNEL,
        &usage_examples.join("\n")
    )
}
