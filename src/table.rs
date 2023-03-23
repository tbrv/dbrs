use crate::row::{Row, ROW_SIZE};

const PAGE_SIZE: usize = 4096;
const TABLE_MAX_PAGES: usize = 100;
const ROWS_PER_PAGE: usize = PAGE_SIZE / ROW_SIZE;

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

    fn add_page(&mut self) {
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

pub struct TableIterator<'a> {
    table: &'a Table,
    position: usize,
}

impl<'a> Iterator for TableIterator<'a> {
    type Item = Row;

    fn next(&mut self) -> Option<Self::Item> {
        if self.position >= self.table.num_rows() {
            return None;
        }
        let row = self.table.select_row(self.position);
        self.position += 1;
        return row;
    }
}

impl Table {
    pub fn iter(&self) -> TableIterator {
        TableIterator {
            table: self,
            position: 0,
        }
    }
}

impl<'a> IntoIterator for &'a Table {
    type Item = Row;
    type IntoIter = TableIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        TableIterator {
            table: &self,
            position: 0,
        }
    }
}


#[cfg(test)]
mod tests {
    use rand::rngs::ThreadRng;
    use rand::distributions::Alphanumeric;
    use rand::Rng;
    use crate::row::{Row, ROW_SIZE};
    use crate::table::{ROWS_PER_PAGE, Table, TABLE_MAX_PAGES};

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


    fn gen_random_string(rng: &mut ThreadRng, max_len: usize) -> String {
        let random_len = rng.gen_range(1..=max_len);
        rng.sample_iter(&Alphanumeric)
            .take(random_len)
            .map(char::from)
            .collect::<String>()
    }

    #[test]
    fn insert_and_select_lots_of_rows() -> Result<(), String> {
        use crate::row::{EMAIL_SIZE, USERNAME_SIZE};

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


    #[test]
    fn test_iterator() -> Result<(), String> {
        let mut table = Table::new();
        let row1 = Row { id: 100, username: "foo".to_string(), email: "bar".to_string() };
        let row2 = Row { id: 200, username: "baz".to_string(), email: "bam".to_string() };

        table.insert_row(&row1)?;
        table.insert_row(&row2)?;

        let mut iter = table.iter();
        assert_eq!(iter.next().unwrap(), row1);
        assert_eq!(iter.next().unwrap(), row2);
        assert!(iter.next().is_none());

        Ok(())
    }

    #[test]
    fn test_into_iterator() -> Result<(), String> {
        let mut table = Table::new();

        // empty table
        for _ in &table {
            assert!(false);
        }

        // table with two items
        let rows = [Row { id: 100, username: "foo".to_string(), email: "bar".to_string() },
            Row { id: 200, username: "baz".to_string(), email: "bam".to_string() }];

        table.insert_row(&rows[0])?;
        table.insert_row(&rows[1])?;

        let mut i = 0;
        for r in &table {
            assert_eq!(r, rows[i]);
            i += 1;
        }

        Ok(())
    }
}