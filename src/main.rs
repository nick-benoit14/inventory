extern crate rusqlite;

use rusqlite::{Connection, MappedRows};
use std::io::{self, Read, Write};

#[derive(Debug)]
struct Item {
    id: Option<i32>,
    name: String,
    necessity: i32,
    quantity: Option<u32>,
}

#[derive(Debug)]
struct Category {
    id: Option<i32>,
    name: String,
}

#[derive(Debug)]
struct CategoryItem {
    id: Option<i32>,
    category_id: i32,
    item_id: i32,
}


fn get_id(conn: &Connection) -> i32{
  let mut last_id_stmt = conn.prepare("SELECT last_insert_rowid() LIMIT 1").unwrap();
  let mut rows = last_id_stmt.query(&[]).unwrap();
  let id: i32 = rows.next().unwrap().unwrap().get(0);
  id
}

fn main() {
    let conn = Connection::open("/Users/nickbenoit/Dropbox/Files/Inventory/inventory.db")
        .unwrap();

    let questions = [
        ("name", "What is the item name?"),
        ("necessity", "What is the item necessity?"),
        ("quantity", "Does the item have a quantity?"),
        ("categories", "What categories does it belong to?")
    ];

    let answers: Vec<(String, String)> = questions.into_iter().map(|q| {
      let mut buffer = String::new();
      io::stdout().write(q.1.as_bytes()).unwrap();
      io::stdout().write("\n".as_bytes()).unwrap();
      io::stdin().read_line(&mut buffer).unwrap();
      (String::from(q.0), buffer.clone())
    }).collect();

    let mut item = Item {
      id: None,
      name: String::from((&answers[0].1).trim()),
      necessity: match answers[1].1.trim().parse::<i32>() {
       Ok(x) => { x },
       Err(_) => { 0 }
      },
      quantity: match answers[2].1.trim().parse::<u32>() {
        Ok(x) => { Some(x) },
        Err(a) => {
            println!("{:?}", a);
            None }
      }
    };

    conn.execute(
        "INSERT INTO items(name, necessity, quantity) VALUES (?1, ?2, ?3)", &[
         &item.name, &item.necessity, &item.quantity
    ]).unwrap();

    item.id = Some(get_id(&conn));

    let mut stmt = conn.prepare("SELECT name FROM categories").unwrap();
    let categories = stmt.query_map(&[], |row| {
        Category {
         id: Some(get_id(&conn)),
         name: row.get(0)
        }
    }).unwrap();

    let mut existing_categories: Vec<String> = Vec::new();
    for c in categories {
        existing_categories.push(c.unwrap().name);
    }

    let given_categories = answers[3].1.trim().split(',').collect::<Vec<&str>>();
    let needed_categories = given_categories.iter().filter(|c| {
      match existing_categories.iter().find(|ec| &ec == c) {
       Some(_) => false,
       None => true
      }
    }).map(|&x| x).collect::<Vec<&str>>();

    let mut created_categories: Vec<Category> = Vec::new();
    for cat in needed_categories {
        conn.execute("INSERT INTO categories(name) VALUES (?1)", &[&cat]).unwrap();
        created_categories.push(Category{
          id: Some(get_id(&conn)),
          name: String::from(cat),
        });
    }

    let mut item_categories: Vec<CategoryItem> = Vec::new();
    for created in created_categories {
     item_categories.push(CategoryItem{
      id: None,
      item_id: item.id.unwrap(),
      category_id: created.id.unwrap()
     });
    }

   for ic in item_categories {
     conn
       .execute("INSERT INTO category_items(item_id, category_id) VALUES (?1, ?2)",
       &[&ic.item_id, &ic.category_id]).unwrap();
   }

}
