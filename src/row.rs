use std::cmp;

#[derive(Debug)]
#[derive(PartialEq)]
pub struct Row {
    pub id: u32,
    pub username: String,
    pub email: String,
}

pub const ID_SIZE: usize = 4;
pub const USERNAME_OFFSET: usize = ID_SIZE;
pub const USERNAME_SIZE: usize = 32;
pub const EMAIL_OFFSET: usize = USERNAME_OFFSET + USERNAME_SIZE;
pub const EMAIL_SIZE: usize = 255;
pub const ROW_SIZE: usize = ID_SIZE + USERNAME_SIZE + EMAIL_SIZE;

impl Row {
    pub fn from_string(s: &str) -> Result<Self, String> {
        let words: Vec<&str> = s.split_whitespace().collect();
        if words.len() != 3 {
            return Err(format!("Expected 3 fields but got {} fields: {}", words.len(), s));
        }
        let id = match words[0].parse::<u32>() {
            Ok(val) => val,
            Err(_) => return Err(String::from("Invalid id")),
        };

        let username = String::from(words[1]);
        let email = String::from(words[2]);
        Ok(Self { id, username, email })
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut buf_array = [0; ROW_SIZE];
        buf_array[..ID_SIZE].copy_from_slice(&self.id.to_le_bytes());

        let username_size = cmp::min(USERNAME_SIZE, self.username.len());
        buf_array[USERNAME_OFFSET..USERNAME_OFFSET + username_size].copy_from_slice(&self.username.as_bytes()[..username_size]);

        let email_size = cmp::min(EMAIL_SIZE, self.email.len());
        buf_array[EMAIL_OFFSET..EMAIL_OFFSET + email_size].copy_from_slice(&self.email.as_bytes()[..email_size]);

        buf_array.to_vec()
    }

    pub fn deserialize(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() != ROW_SIZE {
            return Err(format!("Expected bytes array of size {} but got {}", ROW_SIZE, bytes.len()));
        }

        let mut id_bytes = [0; ID_SIZE];
        id_bytes.copy_from_slice(&bytes[0..ID_SIZE]);
        let id = u32::from_le_bytes(id_bytes);

        let username_bytes = &bytes[USERNAME_OFFSET..USERNAME_OFFSET + USERNAME_SIZE];
        let username_end = get_nul_position(&username_bytes);

        let username = std::str::from_utf8(&username_bytes[..username_end])
            .map_err(|e| e.to_string())?.to_string();

        let email_bytes = &bytes[EMAIL_OFFSET..EMAIL_OFFSET + EMAIL_SIZE];
        let email_end = get_nul_position(&email_bytes);

        let email = std::str::from_utf8(&email_bytes[..email_end])
            .map_err(|e| e.to_string())?.to_string();

        Ok(Row { id, username, email })
    }
}

fn get_nul_position(str_bytes: &[u8]) -> usize {
    str_bytes.iter()
        .position(|&c| c == b'\0')
        .unwrap_or(str_bytes.len())
}
