#[cfg(test)]
mod tests {
    use akinator_rs::Akinator;
    use akinator_rs::enums::{Theme, Answer};
    use akinator_rs::error::{Result, Error};

    #[tokio::test]
    /// a complete test for a full akinator game
    async fn test_akinator() -> Result<()> {
        // create a new akinator instance
        let mut akinator = Akinator::new()
            .with_theme(Theme::Objects) // set theme to objects
            .with_child_mode(); // set child mode to true

        // start the akinator game
        let first_question = akinator.start().await?;

        // print out our first question
        println!("{}",
            first_question.unwrap_or_else(|| "no question".to_string())
        );

        // while the progression of the akinator is less than 80,
        // we keep requesting for questions and answering them.
        while akinator.progression <= 80.0 {
            // the string buffer to store the console input
            let mut answer = String::new();

            // reads answer from standard (console) input
            std::io::stdin()
                .read_line(&mut answer)
                .expect("Failed to read input from console");

            match answer.as_str() {
                // if user response is "back"
                "back" => {
                    // we go back one question with `Akinator::back`
                    match akinator.back().await {
                        // Ok Result from `back`: we print out the next question
                        Ok(question) => println!("{}",
                            question.unwrap_or_else(|| "no question".to_string())
                        ),
                        // Can't go back any further, we are already on the first question most likely
                        Err(Error::CantGoBackAnyFurther) => println!("Cannot go back anymore!"),
                        // Something else went wrong
                        Err(_) => println!("Something else went wrong.")
                    }
                },
                // Any other response from the user
                other => {
                    // Valid answer from user
                    if let Ok(ans) = other.parse::<Answer>() {
                        // answers the akinator with the answer
                        let question = akinator.answer(ans).await?;

                        // print out the next question
                        println!("{}",
                            question.unwrap_or_else(|| "no question".to_string())
                        );
                    } else {
                        // Invalid answer from user
                        println!("Invalid Answer");
                    }
                }
            }
        }

        // akinator progression is at or over 80
        // we then tell the akinator to end the game and make its guess with `Akinator::win`
        let first_guess = akinator.win().await?;

        if let Some(guess) = first_guess {
            println!("Game Over!\n");
            // print its first guess's name
            println!("NAME: {}", guess.name);
            // print its first guess's description
            println!("DESCRIPTION: {}", guess.description);
            // print its first guess's image URL
            println!("IMAGE URL: {}", guess.absolute_picture_path);
        } else {
            println!("no guess from the akinator");
        }

        Ok(())
    }
}