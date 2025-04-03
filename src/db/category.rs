use anyhow::Result;
use rusqlite::{params, Connection};
use crate::models::category::{Category, Rule};

pub struct CategoryDb<'a> {
    conn: &'a mut Connection,
}

impl<'a> CategoryDb<'a> {
    pub fn new(conn: &'a mut Connection) -> Self {
        Self { conn }
    }

    pub fn save_category(&mut self, category: &Category) -> Result<i64> {
        let tx = self.conn.transaction()?;
        
        tx.execute(
            "INSERT INTO categories (name) VALUES (?)",
            params![category.name],
        )?;
        
        let category_id = tx.last_insert_rowid();
        
        for rule in &category.rules {
            tx.execute(
                "INSERT INTO category_rules (category_id, pattern, priority) VALUES (?, ?, ?)",
                params![category_id, rule.pattern, rule.priority],
            )?;
        }
        
        tx.commit()?;
        Ok(category_id)
    }

    pub fn get_all_categories(&mut self) -> Result<Vec<Category>> {
        let mut stmt = self.conn.prepare(
            "SELECT c.id, c.name, cr.pattern, cr.priority 
             FROM categories c 
             LEFT JOIN category_rules cr ON c.id = cr.category_id"
        )?;

        let rows = stmt.query_map([], |row| {
            let id: i64 = row.get(0)?;
            let name: String = row.get(1)?;
            let pattern: Option<String> = row.get(2).ok();
            let priority: Option<u8> = row.get(3).ok();

            Ok((id, name, pattern, priority))
        })?;

        let mut categories = Vec::new();
        let mut current_category: Option<(i64, Category)> = None;

        for row in rows {
            let (id, name, pattern, priority) = row?;

            if let Some((current_id, _)) = current_category.as_ref() {
                if *current_id != id {
                    if let Some((_, category)) = current_category.take() {
                        categories.push(category);
                    }
                }
            }

            if current_category.is_none() {
                current_category = Some((id, Category {
                    name,
                    rules: Vec::new(),
                }));
            }

            if let (Some(pattern), Some(priority)) = (pattern, priority) {
                if let Some((_, category)) = current_category.as_mut() {
                    category.rules.push(Rule {
                        pattern,
                        category: category.name.clone(),
                        priority,
                    });
                }
            }
        }

        if let Some((_, category)) = current_category {
            categories.push(category);
        }

        Ok(categories)
    }

    pub fn assign_category(&mut self, transaction_id: i64, category_id: i64) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO transaction_categories (transaction_id, category_id)
             VALUES (?, ?)",
            params![transaction_id, category_id],
        )?;
        Ok(())
    }

    pub fn initialize_default_categories(&mut self) -> Result<()> {
        let categories = Category::default_categories();
        for category in categories {
            self.save_category(&category)?;
        }
        Ok(())
    }

    pub fn get_category_by_name(&mut self, name: &str) -> Result<Option<Category>> {
        let mut stmt = self.conn.prepare(
            "SELECT c.id, c.name, cr.pattern, cr.priority 
             FROM categories c 
             LEFT JOIN category_rules cr ON c.id = cr.category_id
             WHERE c.name = ?"
        )?;

        let rows = stmt.query_map(params![name], |row| {
            let id: i64 = row.get(0)?;
            let name: String = row.get(1)?;
            let pattern: Option<String> = row.get(2).ok();
            let priority: Option<u8> = row.get(3).ok();

            Ok((id, name, pattern, priority))
        })?;

        let mut category: Option<Category> = None;

        for row in rows {
            let (_, name, pattern, priority) = row?;

            if category.is_none() {
                category = Some(Category {
                    name,
                    rules: Vec::new(),
                });
            }

            if let (Some(pattern), Some(priority)) = (pattern, priority) {
                if let Some(category) = category.as_mut() {
                    category.rules.push(Rule {
                        pattern,
                        category: category.name.clone(),
                        priority,
                    });
                }
            }
        }

        Ok(category)
    }
}