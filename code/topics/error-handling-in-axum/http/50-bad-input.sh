# serve: -p error-handling-in-axum --example errors-bad-input
curl -i -X POST http://127.0.0.1:3000/books -H 'content-type: application/json' -d '{"title":"   "}'
curl -i -X POST http://127.0.0.1:3000/books -H 'content-type: application/json' -d '{"title":"Hamlet"}'
