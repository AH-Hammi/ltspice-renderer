#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pin {
    position: (i32, i32),
    name: Option<String>,
    spice_order: Option<i32>,
}

impl Pin {
    pub fn from_asc_line(line: &str) -> Option<Self> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 3 {
            return None;
        }

        let x = parts[1].parse::<i32>().ok()?;
        let y = parts[2].parse::<i32>().ok()?;

        let name = if parts.len() > 3 {
            Some(parts[3].to_string())
        } else {
            None
        };

        let spice_order = if parts.len() > 4 {
            parts[4].parse::<i32>().ok()
        } else {
            None
        };

        Some(Pin {
            position: (x, y),
            name,
            spice_order,
        })
    }

    pub fn add_attribute(&mut self, line: &str) -> Result<(), String> {
        // Verify the line starts with PINATTR
        if !line.starts_with("PINATTR") {
            return Err("Line does not start with PINATTR".to_string());
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 3 {
            return Err("Not enough parts in line".to_string());
        }

        match parts[1] {
            "PinName" => {
                self.name = Some(parts[2].to_string());
            }
            "SpiceOrder" => {
                if let Ok(order) = parts[2].parse::<i32>() {
                    self.spice_order = Some(order);
                }
            }
            _ => {}
        }
        Ok(())
    }
}

pub struct AsyFile {
    pub pins: Vec<Pin>,
}
