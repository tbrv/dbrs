use std::cmp;

use rand::distributions::Alphanumeric;
use rand::Rng;
use rand::rngs::ThreadRng;

#[derive(Debug)]
#[derive(PartialEq)]
pub struct Row {
    pub id: u32,
    pub username: String,
    pub email: String,
}

const ID_SIZE: usize = 4;
const USERNAME_OFFSET: usize = ID_SIZE;
const USERNAME_SIZE: usize = 32;
const EMAIL_OFFSET: usize = USERNAME_OFFSET + USERNAME_SIZE;
const EMAIL_SIZE: usize = 255;
const ROW_SIZE: usize = ID_SIZE + USERNAME_SIZE + EMAIL_SIZE;

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

#[test]
fn test_row_from_string_happy_case() -> Result<(), String> {
    let row_str = "1 john john@example.com";
    let expected_row = Row { id: 1, username: String::from("john"), email: String::from("john@example.com") };
    let actual_row = Row::from_string(row_str)?;
    assert_eq!(expected_row, actual_row);
    Ok(())
}

#[test]
fn test_row_from_string_wrong_number_of_components() -> Result<(), String> {
    let row_str = "1 john";
    Row::from_string(row_str).unwrap_err();
    Ok(())
}

#[test]
fn test_row_from_string_malformed_string() -> Result<(), String> {
    let row_strings = [
        "one john foo@bar",
        "999999999999999999 foo foo@bar",
        "1", ""
    ];
    for row_str in row_strings.iter() {
        Row::from_string(row_str).unwrap_err();
    }
    Ok(())
}

#[test]
fn serialize_and_deserialize() -> Result<(), String> {
    let original_row = Row {
        id: 10,
        username: String::from("some string"),
        email: String::from("foo@bar.com"),
    };

    let serialized = original_row.serialize();
    let deserialized_row = Row::deserialize(&serialized)?;

    assert_eq!(original_row, deserialized_row);

    Ok(())
}


///////////////////////////////////

const PAGE_SIZE: usize = 4096;
const TABLE_MAX_PAGES: usize = 100;
const ROWS_PER_PAGE: usize = PAGE_SIZE / ROW_SIZE;
const TABLE_MAX_ROWS: usize = ROWS_PER_PAGE * TABLE_MAX_PAGES;

type Page = [u8; PAGE_SIZE];

#[derive(Debug)]
pub struct Table {
    pages: Vec<Page>,
    num_rows: usize,
}

impl Table {
    pub fn new() -> Self {
        Table {
            pages: Vec::new(),
            num_rows: 0,
        }
    }

    pub fn num_pages(&self) -> usize {
        self.pages.len()
    }

    pub fn num_rows(&self) -> usize {
        self.num_rows
    }

    pub fn add_page(&mut self) {
        self.pages.push([0; PAGE_SIZE]);
    }

    pub fn insert_row(&mut self, row: &Row) -> Result<(), String> {
        let (page_num, byte_offset_in_page) = Table::row_position(self.num_rows);

        if page_num > TABLE_MAX_PAGES {
            return Err(String::from("Reached max number of pages"));
        } else if page_num >= self.pages.len() {
            self.add_page();
        }

        let page = self.pages.get_mut(page_num).unwrap();

        let row_bytes = row.serialize();
        for (i, b) in row_bytes.iter().enumerate() {
            page[byte_offset_in_page + i] = *b;
        }
        self.num_rows += 1;

        Ok(())
    }

    /// Returns the page and the byte-offset in page for a given row number
    fn row_position(row_num: usize) -> (usize, usize) {
        let page_num = row_num / ROWS_PER_PAGE;
        let row_in_page = row_num % ROWS_PER_PAGE;
        let byte_offset_in_page = row_in_page * ROW_SIZE;
        (page_num, byte_offset_in_page)
    }

    pub fn select_row(&self, position: usize) -> Option<Row> {
        let (page_num, byte_offset_in_page) = Table::row_position(position);
        if page_num >= self.pages.len() {
            return None;
        }
        let page = self.pages.get(page_num).unwrap();
        let bytes = &page[byte_offset_in_page..byte_offset_in_page + ROW_SIZE];
        let row = Row::deserialize(bytes);

        Some(row.unwrap())
    }
}

#[test]
fn row_position() -> Result<(), String> {
    assert_eq!(Table::row_position(0), (0, 0));
    assert_eq!(Table::row_position(ROWS_PER_PAGE), (1, 0));
    assert_eq!(Table::row_position(ROWS_PER_PAGE + 10), (1, 10 * ROW_SIZE));
    assert_eq!(Table::row_position((TABLE_MAX_PAGES + 1) * ROWS_PER_PAGE), (TABLE_MAX_PAGES + 1, 0));

    Ok(())
}

#[test]
fn insert_row_and_select() -> Result<(), String> {
    let mut table = Table::new();
    let row = Row {
        id: 100,
        username: String::from("Yz85rmUs0CzYJBDDA6hY38I07uOq6u2R"),
        email: String::from("qopa@apoq.com"),
    };

    table.insert_row(&row).expect("no error");

    let page = table.pages.get(0);
    assert!(page.is_some());
    assert_eq!(table.num_rows(), 1);

    let row_from_table = table.select_row(0).unwrap();
    assert_eq!(row, row_from_table);

    Ok(())
}

#[test]
fn insert_and_select_multiple_rows() -> Result<(), String> {
    let rows = [
        Row::from_string("10 Andrew andre.jung@gmail.com")?,
        Row::from_string("30 Birte birte.hochlander@web.de")?,
        Row::from_string("20 Yanik yk@nomail.com")?,
    ];

    let mut table = Table::new();
    for row in rows.iter() {
        table.insert_row(row)?;
    }

    for i in 0..table.num_rows() {
        let table_row = table.select_row(i).unwrap();
        assert_eq!(rows[i], table_row);
    }

    Ok(())
}

#[test]
fn insert_and_select_lots_of_rows() -> Result<(), String> {
    let mut rng = rand::thread_rng();
    let num_rows = 1000;
    let mut rows: Vec<Row> = Vec::new();
    let mut table = Table::new();

    for _i in 0..num_rows {
        let random_id = rng.gen();
        let random_name = gen_random_string(&mut rng, USERNAME_SIZE);
        let random_email = gen_random_string(&mut rng, EMAIL_SIZE);

        let row = Row { id: random_id, username: random_name, email: random_email };

        table.insert_row(&row)?;
        rows.push(row);
    }

    for i in 0..table.num_rows() {
        let table_row = table.select_row(i).unwrap();
        assert_eq!(rows[i], table_row);
    }

    Ok(())
}

#[cfg(test)]
fn gen_random_string(rng: &mut ThreadRng, max_len: usize) -> String {
    let random_len = rng.gen_range(1..=USERNAME_SIZE);
    rng.sample_iter(&Alphanumeric)
        .take(random_len)
        .map(char::from)
        .collect::<String>()
}
