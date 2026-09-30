# serve: -p error-handling-in-axum --example errors-delete
curl -i -X DELETE http://127.0.0.1:3000/books/2
curl -i http://127.0.0.1:3000/books/2
curl -i -X DELETE http://127.0.0.1:3000/books/2
curl -i http://127.0.0.1:3000/books/1
