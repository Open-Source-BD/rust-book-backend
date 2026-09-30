# serve: -p json-and-serde --example json-camel-case
curl -i http://127.0.0.1:3000/books/sample
curl -i -X POST http://127.0.0.1:3000/books -H 'content-type: application/json' -d '{"title":"Emma","inStock":true}'
curl -i -X POST http://127.0.0.1:3000/books -H 'content-type: application/json' -d '{"title":"Emma","in_stock":true}'
