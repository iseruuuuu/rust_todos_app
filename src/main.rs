use warp::Filter;
#[tokio::main]
async fn main() {
    // GET /hello/warp => 200 OK with body "Hello, warp!"
    // let hello = warp::path!("hello" / String).map(|name| format!("Hello, {}!", name));
    // warp::serve(hello).run(([127, 0, 0, 1], 3030)).await;

    let db = models::init_todos(); //todoデータ読み込み
    let routes = filters::todos(db); //ルーティング＋handlers

    //サーバ起動
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;

}
//ルーティング定義
mod filters {
    use super::handlers;
    use super::models::{Db, Todo};

    use warp::Filter;

    //すべてのルーティングを束ねて返却する
    pub fn todos(
        db: Db,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        //以下にルーティング定義を足していく
        todos_list(db.clone())
            .or(todos_create(db.clone()))
            .or(todos_delete(db.clone()))
            .or(todos_update(db.clone()))
    }

    //GET /todos => 一覧でtodoリスト返却
    pub fn todos_list(
        db: Db,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("todos")
            .and(warp::get())
            .and(with_db(db))
            .and_then(handlers::list_todos)
    }
    //dbとの接続
    fn with_db(db: Db) -> impl Filter<Extract = (Db,), Error = std::convert::Infallible> + Clone {
        warp::any().map(move || db.clone())
    }

    //POST /todos => 新規Todoの登録
    pub fn todos_create(
        db: Db,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("todos")
            .and(warp::post())
            .and(decode_json())
            .and(with_db(db))
            .and_then(handlers::create_todo)
    }

    //jsonの読み取り
    fn decode_json() -> impl Filter<Extract = (Todo,), Error = warp::Rejection> + Clone {
        warp::body::content_length_limit(1024 * 16).and(warp::body::json())
    }

    // PUT /todos/:id => id指定でtodo更新
    pub fn todos_update(
        db: Db,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("todos" / u64)
            .and(warp::put())
            .and(decode_json())
            .and(with_db(db))
            .and_then(handlers::update_todo)
    }

    // DELETE /todos/:id　=> id指定でtodo削除
    pub fn todos_delete(
        db: Db,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("todos" / u64)
            .and(warp::delete())
            .and(with_db(db))
            .and_then(handlers::delete_todo)
    }
}

mod handlers {
    use super::models::{Db, Todo};
    use std::convert::Infallible;
    use warp::http::StatusCode;

    //Todoリストを返却
    pub async fn list_todos(db: Db) -> Result<impl warp::Reply, Infallible> {
        let todos = db.lock().await;
        let todos: Vec<Todo> = todos.clone().into_iter().collect();

        Ok(warp::reply::json(&todos))
    }
    //Todoリストに新規追加
    pub async fn create_todo(create: Todo, db: Db) -> Result<impl warp::Reply, Infallible> {
        let mut vec = db.lock().await;
        //idが重複していた場合弾く
        for todo in vec.iter() {
            if todo.id == create.id {
                return Ok(StatusCode::BAD_REQUEST);
            }
        }

        vec.push(create);
        Ok(StatusCode::CREATED)
    }

    //Todo更新
    pub async fn update_todo(
        id: u64,
        update: Todo,
        db: Db,
    ) -> Result<impl warp::Reply, Infallible> {
        let mut vec = db.lock().await;

        for todo in vec.iter_mut() {
            if todo.id == id {
                *todo = update;
                return Ok(StatusCode::OK);
            }
        }

        Ok(StatusCode::NOT_FOUND)
    }

    //Todo削除
    pub async fn delete_todo(id: u64, db: Db) -> Result<impl warp::Reply, Infallible> {
        let mut vec = db.lock().await;

        let len = vec.len();
        vec.retain(|todo| todo.id != id);

        let deleted = vec.len() != len;

        if deleted {
            Ok(StatusCode::NO_CONTENT)
        } else {
            Ok(StatusCode::NOT_FOUND)
        }
    }
}

mod models {
    // dbからToDOリストを取得する
    use serde_derive::{Deserialize, Serialize};
    use std::sync::Arc;
    use tokio::sync::Mutex;

    //Todo構造体、各フィールド定義を記述して置くことができます
    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct Todo {
        pub id: u64,
        pub text: String,
        pub completed: bool,
    }
    //dbの型定義、Arcはマルチスレッドでスレッド間で共有できるオブジェクトを提供します
    pub type Db = Arc<Mutex<Vec<Todo>>>;

    //todoリストの初期化
    pub fn init_todos() -> Db {
        //テスト　初期表示確認用
        let test = Todo {
            id: 1,
            text: "TODOの内容について".into(),
            completed: false,
        };
        Arc::new(Mutex::new(Vec::from([test])))
    }
}
