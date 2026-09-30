# serve: -p error-handling-in-axum --example errors-gone
curl -i http://127.0.0.1:3000/books/13
curl -i http://127.0.0.1:3000/books/9
curl -i http://127.0.0.1:3000/books/2
