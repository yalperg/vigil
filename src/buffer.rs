pub struct Buffer {
    pub file: Option<String>,
    pub lines: Vec<String>,
}

impl Buffer {
    pub fn from_file(file: Option<String>) -> Self {
        let lines = match &file {
            Some(file) => std::fs::read_to_string(file)
                .unwrap()
                .lines()
                .map(|line| line.to_string())
                .collect(),
            None => vec![],
        };

        Self { file, lines }
    }

    pub fn get(&self, line: usize) -> Option<String> {
        if self.lines.len() > line {
            return Some(self.lines[line].clone());
        }

        None
    }

    pub fn len(&self) -> usize {
        self.lines.len()
    }

    pub fn insert(&mut self, x: u16, y: u16, c: char) {
        let y = y as usize;
        if let Some(line) = self.lines.get_mut(y) {
            let mut new_line = String::new();
            let mut char_count = 0;
            let x = x as usize;
    
            for ch in line.chars() {
                if char_count == x {
                    new_line.push(c);
                }
                new_line.push(ch);
                char_count += 1;
            }
    
            if char_count < x {
                new_line.push_str(&" ".repeat(x - char_count));
                new_line.push(c);
            } else if char_count == x {
                new_line.push(c);
            }
    
            *line = new_line;
        } else {
            let mut new_line = String::new();
            if x > 0 {
                new_line.push_str(&" ".repeat(x as usize));
            }
            new_line.push(c);
            self.lines.push(new_line);
        }
    }

    pub fn remove(&mut self, x: u16, y: u16) {
        let y = y as usize;
        let x = x as usize;
    
        if let Some(line) = self.lines.get_mut(y) {
            if !line.is_empty() && x < line.len() {
                line.remove(x);
            }
        }
    }

    pub fn save(&self) {
        if let Some(file) = &self.file {
            let content = self.lines.join("\n");
            std::fs::write(file, content).unwrap();
        }
    }
}
