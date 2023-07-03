use std::{vec, collections::HashMap};
use rand::{thread_rng, seq::SliceRandom};

#[derive(Debug)]
pub struct Names {
    possible_generated_names: Vec<String>,
    taken_names: HashMap<String, usize>,
}

impl Names {
    pub fn new() -> Self {
        let colors = vec!["Red", "Blue", "Yellow", "Green", "Purple", "Cyan", "Magenta", "Pink"];
        let animals = vec!["Penguin", "Deer", "Ostrich", "Giraffe", "Elephant", "Dolphin", "Cat"];
        let mut possible_generated_names : Vec<String> = vec![];
        for color in colors {
            for animal in &animals {
                possible_generated_names.push(format!("{color} {animal}"));
                possible_generated_names.shuffle(&mut thread_rng())
            }
        }

        Self { possible_generated_names, taken_names:HashMap::new() }
    }

    // Returns the given name if the name hasn't been taken, otherwise returns a 'variant' of the name
    // eg: if John is taken, it will return John#2
    fn validate_and_register_name(&mut self, name: String) -> String {
        match self.taken_names.get(&name) {
            Some(value) => {
                let new_name = format!("{}#{}", name, value+1);
                self.taken_names.insert(name, value+1);
                new_name
            }
            None => {
                self.taken_names.insert(name.clone(), 1);
                name
            }
        }
    }

    pub fn get_random_name(&mut self) -> String {
        // TODO: This sets a bound to the number of players who can play this game
        let mut name = self.possible_generated_names.pop().unwrap(); 
        name = self.validate_and_register_name(name);
        name
    }

    pub fn get_specific_name(&mut self, name : String) -> Result<String, ()> {
        if name.contains('#') {
            return Err(());
        }
        let ret_name = self.validate_and_register_name(name);
        Ok(ret_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_names() {
        let mut names = Names::new();
        let name1 = names.get_specific_name("Shelly".to_string()).unwrap();
        assert_eq!(name1, "Shelly".to_string());
        let name2 = names.get_specific_name("Shelly".to_string()).unwrap();
        assert_eq!(name2, "Shelly#2".to_string());
        let name3 = names.get_specific_name("Shelly".to_string()).unwrap();
        assert_eq!(name3, "Shelly#3".to_string());
        let name4 = names.get_specific_name("Shellyy".to_string()).unwrap();
        assert_eq!(name4, "Shellyy".to_string());
    }

    #[test]
    fn invalid_name() {
        let mut names = Names::new();
        let name1 = names.get_specific_name("StaticESC#1234".to_string());
        assert!(name1.is_err());
    }
}

