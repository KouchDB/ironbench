README

This is a simple Rust program that provides an HTTP API to store and retrieve JSON documents in a file-based database. The program uses the Hyper crate to implement the HTTP server and the Rusqlite crate to store the document index in an SQLite database.

To run the program, execute cargo run in the terminal. The program listens on port 3000 by default, but this can be changed by modifying the addr variable in the main function.

When the server is running, it accepts GET and PUT requests to the /db/{id} endpoint. The {id} parameter specifies the ID of the document to get or put. The body of a PUT request should be a JSON representation of the document.

The program uses the handle_request function to handle incoming requests. This function extracts the method and path from the request and dispatches the request to the appropriate handler function.

The Database struct represents the file-based database. It has methods to get and put documents in the database. When a document is put in the database, its ID and offset in the file are stored in the index database.

The program uses the Tokio runtime to handle asynchronous I/O operations. The tokio::main macro is used to start the runtime and run the main function.

The program's data model and API are compatible with CouchDB, which is a popular document-oriented NoSQL database. CouchDB stores documents as JSON objects and provides a RESTful HTTP API to create, read, update, and delete documents. The program stores documents in a file-based database and provides a simple HTTP API to get and put documents by ID. The program uses the same document model as CouchDB, which has an _id field that uniquely identifies the document and a _rev field that represents the revision of the document. The program also stores the document's fields as a hash map, which is compatible with CouchDB's document model. The program's API is also compatible with CouchDB's API.

The index is stored in an SQLite database using the rusqlite crate. SQLite is a lightweight, file-based relational database management system that provides a SQL interface to manage data. The Database struct contains a Connection object that represents the connection to the SQLite index database. The index database has a single table named index with two columns: id and offset. The id column is a text column that stores the ID of the document, and the offset column is an integer column that stores the offset of the document in the file-based database. When a document is put in the file-based database, its ID and offset are inserted or updated in the index table using an INSERT OR REPLACE SQL statement. When a document is requested from the file-based database, its offset is retrieved from the index table using a SELECT SQL statement with the document ID as the parameter.

Overall, the program's data model and API are simple and compatible with CouchDB, making it a good choice for simple document-oriented applications that need to store data in a file-based database.