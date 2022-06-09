use warp::Filter;
#[tokio::main]
async fn main() {
    // GET /hello/warp => 200 OK with body "Hello, warp!"
    let hello = warp::path!("hello" / String).map(|name| format!("Hello, {}!", name));

    //warp::serve(hello)でサーバ起動
    warp::serve(hello).run(([127, 0, 0, 1], 3030)).await;

    //ビルド方法
    //cargo run
    //http://localhost:3030/hello/fabeee　を開く
}

mod filters {
    use super::handlers;
    use super::models::Db;
    use warp::Filter;

    pub fn todos(db: Db) -> impl Filter + Clone {
        todos_list(db.clone())
    }

    pub fn todos_list(db: Db) -> impl Filter + Clone {
        warp::path!("todos")
            .and(warp::get())
            .and(with_db(db))
            .and_then(handlers::list_todos)
    }

    fn with_db(db: Db) -> impl Filter + Clone {
        warp::any().map(move || db.clone())
    }
}

mod handlers {
    use super::models::{Db, Todo};
    use std::convert::Infallible;

    //Todoリストを返却
    pub async fn list_todos(db: Db) -> Result {
        let todos = db.lock().await;
        let todos: Vec = todos.clone().into_iter().collect();

        Ok(warp::reply::json(&todos))
    }
}

mod models {
    use serde_derive::{Deserialize, Serialize};
    use std::sync::Arc;
    use tokio::sync::Mutex;

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct Todo {
        pub id: u64,
        pub text: String,
        pub completed: bool,
    }

    pub type Db = Arc<Mutex<Vec<Todo>>>;

    pub fn init_todos() -> Db {
        let test = Todo {
            id: 1,
            text: "TODOの内容について".into(),
            completed: false,
        };

        Arc::new(Mutex::new(Vec::from([test])))
    }
}

//https://fabeee.co.jp/column/employee-blog/waka_01/
