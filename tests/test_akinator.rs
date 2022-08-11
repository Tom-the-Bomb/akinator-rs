#[cfg(test)]
mod tests {
    use std::io::stdin;
    use akinator_rs::Akinator;
    use akinator_rs::enums::Answer;
    use akinator_rs::error::Result;

    #[test]
    fn test_akinator() -> Result<()> {
        let mut akinator = Akinator::new();
        akinator.start()?;

        println!("{}",
            akinator.current_question.clone()
                .unwrap_or_else(|| "no question".to_string())
        );

        while akinator.progression <= 80.0 {
            let mut answer = String::new();
            stdin().read_line(&mut answer).unwrap();

            match answer.as_str() {
                "back" => {
                    if let Ok(q) = akinator.back() {
                        println!("{}",
                            q.unwrap_or_else(|| "no question".to_string())
                        );
                    } else {
                        println!("Cannot go back anymore!");
                    }
                },
                other => {
                    if let Ok(ans) = other.parse::<Answer>() {
                        let question = akinator.answer(ans)?;

                        println!("{}",
                            question.unwrap_or_else(|| "no question".to_string())
                        );
                    } else {
                        println!("Invalid Answer");
                    }
                }
            }
        }

        let guess = akinator.win()?.unwrap();
        println!("{}", guess.name);
        println!("{}", guess.description);

        Ok(())
    }
}