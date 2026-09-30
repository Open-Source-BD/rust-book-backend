# serve: -p error-handling-in-axum --example errors-json-rejection
curl -i http://127.0.0.1:3000/books/abc
curl -i -X POST http://127.0.0.1:3000/books -H 'content-type: application/json' -d '{"title":"Emma"'
curl -i -X POST http://127.0.0.1:3000/books -H 'content-type: application/json' -d '{}'
curl -i -X POST http://127.0.0.1:3000/books -d '{"title":"Hamlet"}'
curl -i -X POST http://127.0.0.1:3000/books -H 'content-type: application/json' -d '{"title":"Hamlet"}'
