use actions::channel_help::*;

const EXAMPLE_MESSAGE: &str = "*Example usage:*

To get the colour named Shadow Green, type:

    Shadow Green

To get the colour named French Lilac, type:

    French Lilac


*Colours avaliable:*
";

#[test]
fn help_message_forms_correctly() {
    let names = vec!["Shadow Green".to_string(), "French Lilac".to_string()];

    let full_message = generate_help_message(&names);

    assert_eq!(
        format!(
            "
{}

{}",
            HELP_MESSAGE_FOR_CHANNEL, EXAMPLE_MESSAGE
        ),
        full_message
    );
}

#[test]
fn message_should_not_fail_on_empty_lists() {
    let names: Vec<String> = vec![];

    let full_message = generate_help_message(&names);

    println!("{}", full_message);
    assert!(full_message.contains("no colours"))
}
