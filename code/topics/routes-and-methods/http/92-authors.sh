# serve: -p routes-and-methods --example routes-authors
curl -i http://127.0.0.1:3000/authors
curl -i -X POST http://127.0.0.1:3000/authors
curl -i http://127.0.0.1:3000/authors/3
curl -i -X DELETE http://127.0.0.1:3000/authors/3
curl -i -X PUT http://127.0.0.1:3000/authors/3
