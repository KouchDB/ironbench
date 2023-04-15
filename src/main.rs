use hyper::body::to_bytes;
use hyper::service::{make_service_fn, service_fn};
use hyper::{ Body, Method, Request, Response, StatusCode };
use hyper::server::{Server, conn::AddrStream};
use rusqlite::{params, Connection, Error };
use serde::{ Deserialize, Serialize };
use std::collections::HashMap;
use std::fs::{ File, OpenOptions };
use std::io::{ self, prelude::*, SeekFrom };
use std::net::SocketAddr;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
struct Document {
    id: String,
    rev: Option<String>,
    #[serde(flatten)]
    fields: HashMap<String, String>,
}

#[derive(Debug)]
struct Database {
    file: File,
    index_conn: Connection,
}

impl Database {
    fn new(data_dir: PathBuf) -> io::Result<Self> {
        let data_file_path = data_dir.join("data.txt");
        let index_file_path = data_dir.join("index.db");
        let file = OpenOptions::new().create(true).read(true).append(true).open(&data_file_path)?;
        let index_conn = Connection::open(&index_file_path).unwrap();
        index_conn.execute(
            "CREATE TABLE IF NOT EXISTS 'index' (id TEXT PRIMARY KEY, offset INTEGER)",
            []
        ).unwrap();
        Ok(Database { file, index_conn })
    }

    fn get_document(&self, id: &str) -> io::Result<Option<Document>> {
        let offset = match self.index_conn.query_row(
            "SELECT offset FROM 'index' WHERE id = ?1",
            params![id],
            |row| Ok(row.get::<usize, i64>(0)? as u64),
        ) {
            Ok(offset) => offset,
            Err(Error::QueryReturnedNoRows) => return Ok(None),
            Err(e) => return Err(io::Error::new(io::ErrorKind::Other, e))
        };
        let mut reader = io::BufReader::new(&self.file);
        reader.seek(SeekFrom::Start(offset))?;
        let mut buf = Vec::new();
        reader.read_until(b'\n', &mut buf)?;
        let document: Document = serde_json::from_slice(&buf)?;
        Ok(Some(document))
    }

    fn put_document(&mut self, document: &Document) -> io::Result<()> {
        let offset = self.file.seek(SeekFrom::End(0))?;
        let line = serde_json::to_string(&document)?;
        let mut buf = line.into_bytes();
        buf.push(b'\n');
        self.file.write_all(&buf)?;
        self.index_conn.execute(
            "INSERT OR REPLACE INTO 'index' (id, offset) VALUES (?1, ?2)",
            params![document.id, offset as i64],
        ).unwrap();
        Ok(())
    }
}

async fn handle_request(
    mut req: Request<Body>
) -> Result<Response<Body>, hyper::http::Error> {
    let database = &mut Database::new(PathBuf::from(".")).unwrap();
    let path = req.uri().path().to_owned();
    let method = &req.method().clone();
    match (method, path) {
        (&Method::GET, path) if path.starts_with("/db/") => {
            let id = &path[4..];
            match database.get_document(id) {
                Ok(Some(document)) => {
                    let body = serde_json::to_string(&document).unwrap();
                    Response::builder()
                        .status(StatusCode::OK)
                        .header("Content-Type", "application/json")
                        .body(Body::from(body))
                }
                Ok(None) => Response::builder().status(StatusCode::NOT_FOUND).body(Body::empty()),
                Err(e) => {
                    eprintln!("database error: {}", e);
                    Response::builder().status(StatusCode::INTERNAL_SERVER_ERROR)
                                       .body(Body::empty())
                }
            }
        }
        (&Method::PUT, path) if path.starts_with("/db/") => {
            let id = &path[4..];
            let body = to_bytes(&mut req.body_mut()).await.unwrap();
            eprintln!("body = {:?}", body);
            let document: Document = serde_json::from_slice(&body).unwrap();
            assert_eq!(document.id, id);
            database.put_document(&document).unwrap();
            let body = serde_json::to_string(&document).unwrap();
            Response::builder()
                .status(StatusCode::CREATED)
                .header("Content-Type", "application/json")
                .body(Body::from(body))
        }
        _ => Response::builder().status(StatusCode::NOT_FOUND).body(Body::empty()),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));

    let make_svc = make_service_fn(|socket: &AddrStream | {
        println!("Connected with {:?}", socket.remote_addr());
        async move {
            Ok::<_, hyper::Error>(service_fn(|req| handle_request(req)))
        }
    });

    println!("Listening on http://{}", addr);

    if let Err(e) = Server::bind(&addr).serve(make_svc).await {
        eprintln!("server error: {}", e)
    }

    Ok(())
}