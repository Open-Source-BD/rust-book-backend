# serve: -p routes-and-methods
curl -i http://127.0.0.1:3000/books
curl -i -X POST http://127.0.0.1:3000/books
curl -i http://127.0.0.1:3000/books/7
curl -i -X PUT http://127.0.0.1:3000/books/7
curl -i -X DELETE http://127.0.0.1:3000/books/7
