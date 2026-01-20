use anyhow::Result;
use dialoguer::Input;

pub fn ask_user_prompt() -> Result<String> {
    let input: String = Input::new()
        .with_prompt("Describe your task")
        .interact_text()?;
    Ok(input)
}

pub fn ask_chat_turn() -> Result<String> {
    let input: String = Input::new().with_prompt("You").interact_text()?;
    Ok(input)
}
