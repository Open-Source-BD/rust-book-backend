# serve: -p json-and-serde
curl -i -X POST http://127.0.0.1:3000/books -H 'content-type: application/json' -d '{"in_stock":true}'
curl -i -X POST http://127.0.0.1:3000/books -H 'content-type: application/json' -d '{"title":"Emma"'
curl -i -X POST http://127.0.0.1:3000/books -d '{"title":"Emma"}'
