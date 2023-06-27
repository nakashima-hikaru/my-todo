#[cfg(test)]
pub(crate) mod test_utils {
    use anyhow::Context;
    use axum::async_trait;

    use crate::repositories::hash_map::test_utils::HashMapRepository;
    use crate::repositories::todos::{CreateTodo, Todo, TodoRepository, UpdateTodo};
    use crate::repositories::RepositoryError;

    #[async_trait]
    impl TodoRepository for HashMapRepository {
        async fn create(&self, payload: CreateTodo) -> anyhow::Result<Todo> {
            let mut store = self.write_store_ref();
            let id = (store.len() + 1) as i32;
            let todo = Todo::new(id, payload.text);
            store.insert(id, todo.clone());
            Ok(todo)
        }

        async fn find(&self, id: i32) -> anyhow::Result<Todo> {
            let store = self.read_store_ref();
            let todo = store
                .get(&id)
                .cloned()
                .ok_or(RepositoryError::NotFound("id".to_string(), id))?;
            Ok(todo)
        }

        async fn all(&self) -> anyhow::Result<Vec<Todo>> {
            Ok(self.read_store_ref().values().cloned().collect())
        }

        async fn update(&self, id: i32, payload: UpdateTodo) -> anyhow::Result<Todo> {
            let mut store = self.write_store_ref();
            let todo = store
                .get(&id)
                .with_context(|| RepositoryError::NotFound("id".to_string(), id))?;
            let text = payload.text.unwrap_or(todo.text.clone());
            let completed = payload.completed.unwrap_or(todo.completed);
            let todo = Todo {
                id,
                text,
                completed,
            };
            store.insert(id, todo.clone());
            Ok(todo)
        }

        async fn delete(&self, id: i32) -> anyhow::Result<()> {
            let mut store = self.write_store_ref();
            store
                .remove(&id)
                .ok_or(RepositoryError::NotFound("id".to_string(), id))?;
            Ok(())
        }
    }

    #[cfg(test)]
    mod tests {
        use crate::repositories::todos::UpdateTodo;

        use super::*;

        #[tokio::test]
        async fn todo_create() {
            let text = "todo text".to_string();
            let id = 1;
            let expected = Todo::new(id, text.clone());

            let repository = HashMapRepository::new();
            let todo = repository.create(CreateTodo { text }).await.unwrap();
            assert_eq!(expected, todo);
        }

        #[tokio::test]
        async fn todo_find() {
            let text = "todo text".to_string();
            let id = 1;
            let expected = Todo::new(id, text.clone());

            let repository = HashMapRepository::new();
            repository
                .create(CreateTodo { text })
                .await
                .expect("failed to create todo");
            let todo = repository.find(id).await.unwrap();
            assert_eq!(expected, todo);
        }

        #[tokio::test]
        async fn todo_all() {
            let text = "todo text".to_string();
            let id = 1;
            let expected = Todo::new(id, text.clone());
            let repository = HashMapRepository::new();
            let _ = repository
                .create(CreateTodo { text })
                .await
                .expect("failed to create todo");
            let todo = repository.all().await.unwrap();
            assert_eq!(vec![expected], todo);
        }

        #[tokio::test]
        async fn todo_update() {
            let text = "todo text".to_string();
            let id = 1;
            let repository = HashMapRepository::new();
            let _ = repository
                .create(CreateTodo { text: text.clone() })
                .await
                .expect("failed to create todo");

            let update_text = "update todo text".to_string();
            let todo = repository
                .update(
                    id,
                    UpdateTodo {
                        text: Some(update_text.clone()),
                        completed: Some(true),
                    },
                )
                .await
                .expect("failed update todo.");
            assert_eq!(
                Todo {
                    id,
                    text: update_text,
                    completed: true,
                },
                todo
            );
        }

        #[tokio::test]
        async fn todo_delete() {
            let text = "todo text".to_string();
            let id = 1;
            let repository = HashMapRepository::new();
            let _ = repository
                .create(CreateTodo { text: text.clone() })
                .await
                .expect("failed to create todo");

            let res = repository.delete(id).await;
            assert!(res.is_ok());
            assert_eq!(repository.read_store_ref().len(), 0);
        }
    }
}
