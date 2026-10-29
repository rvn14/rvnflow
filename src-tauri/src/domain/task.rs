use crate::error::{Result, invalid};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    pub id: String,
    pub list_id: String,
    pub title: String,
    pub completed: bool,
    pub created_at: u64,
    pub updated_at: u64,
    pub completed_at: Option<u64>,
    pub order: u32,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskList {
    pub id: String,
    pub name: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tasks {
    pub lists: Vec<TaskList>,
    pub tasks: Vec<Task>,
    pub active_list_id: String,
}
pub fn title(value: &str) -> Result<String> {
    let value = value.trim();
    if value.is_empty() || value.chars().count() > 200 {
        return Err(invalid("Enter a name between 1 and 200 characters."));
    }
    Ok(value.into())
}
impl Default for Tasks {
    fn default() -> Self {
        let id = Uuid::new_v4().to_string();
        Self {
            lists: vec![TaskList {
                id: id.clone(),
                name: "Today".into(),
            }],
            tasks: vec![],
            active_list_id: id,
        }
    }
}
impl Tasks {
    pub fn validate(&self) -> Result<()> {
        let ids: HashSet<_> = self.lists.iter().map(|l| &l.id).collect();
        let tasks: HashSet<_> = self.tasks.iter().map(|t| &t.id).collect();
        if self.lists.is_empty()
            || ids.len() != self.lists.len()
            || tasks.len() != self.tasks.len()
            || !ids.contains(&self.active_list_id)
            || self
                .tasks
                .iter()
                .any(|t| !ids.contains(&t.list_id) || t.id.is_empty())
        {
            return Err(invalid("The saved task lists contain invalid references."));
        }
        for list in &self.lists {
            title(&list.name)?;
            if list.id.is_empty() {
                return Err(invalid("Invalid list ID."));
            }
        }
        for task in &self.tasks {
            title(&task.title)?;
        }
        Ok(())
    }
    pub fn create_list(&mut self, name: &str) -> Result<String> {
        let id = Uuid::new_v4().to_string();
        self.lists.push(TaskList {
            id: id.clone(),
            name: title(name)?,
        });
        self.active_list_id = id.clone();
        Ok(id)
    }
    pub fn rename_list(&mut self, id: &str, name: &str) -> Result<()> {
        let name = title(name)?;
        self.lists
            .iter_mut()
            .find(|l| l.id == id)
            .ok_or_else(|| invalid("List not found."))?
            .name = name;
        Ok(())
    }
    pub fn delete_list(&mut self, id: &str) -> Result<()> {
        if self.lists.len() == 1 {
            return Err(invalid("Keep at least one task list."));
        }
        if !self.lists.iter().any(|l| l.id == id) {
            return Err(invalid("List not found."));
        }
        self.lists.retain(|l| l.id != id);
        self.tasks.retain(|t| t.list_id != id);
        if self.active_list_id == id {
            self.active_list_id = self.lists[0].id.clone();
        }
        Ok(())
    }
    pub fn create(&mut self, list_id: &str, name: &str, now: u64) -> Result<String> {
        if !self.lists.iter().any(|l| l.id == list_id) {
            return Err(invalid("List not found."));
        }
        let name = title(name)?;
        let id = Uuid::new_v4().to_string();
        let order = self
            .tasks
            .iter()
            .filter(|t| t.list_id == list_id)
            .map(|t| t.order)
            .max()
            .map_or(0, |v| v.saturating_add(1));
        self.tasks.push(Task {
            id: id.clone(),
            list_id: list_id.into(),
            title: name,
            completed: false,
            created_at: now,
            updated_at: now,
            completed_at: None,
            order,
        });
        Ok(id)
    }
    pub fn update(&mut self, id: &str, name: Option<&str>, toggle: bool, now: u64) -> Result<()> {
        let name = name.map(title).transpose()?;
        let task = self
            .tasks
            .iter_mut()
            .find(|t| t.id == id)
            .ok_or_else(|| invalid("Task not found."))?;
        if let Some(name) = name {
            task.title = name;
        }
        if toggle {
            task.completed = !task.completed;
            task.completed_at = task.completed.then_some(now);
        }
        task.updated_at = now;
        Ok(())
    }
    pub fn delete(&mut self, id: &str) -> Result<()> {
        if !self.tasks.iter().any(|t| t.id == id) {
            return Err(invalid("Task not found."));
        }
        self.tasks.retain(|t| t.id != id);
        Ok(())
    }
    pub fn reorder(&mut self, list_id: &str, ids: &[String]) -> Result<()> {
        let expected: HashSet<_> = self
            .tasks
            .iter()
            .filter(|t| t.list_id == list_id)
            .map(|t| t.id.as_str())
            .collect();
        let actual: HashSet<_> = ids.iter().map(String::as_str).collect();
        if actual != expected || actual.len() != ids.len() {
            return Err(invalid("The task order is out of date. Please try again."));
        }
        for (order, id) in ids.iter().enumerate() {
            if let Some(task) = self.tasks.iter_mut().find(|t| t.id == *id) {
                task.order = order as u32;
            }
        }
        Ok(())
    }
}
