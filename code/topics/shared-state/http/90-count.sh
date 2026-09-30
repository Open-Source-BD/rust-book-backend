# serve: -p shared-state --example state-count
curl -i http://127.0.0.1:3000/books/count
curl -i -X POST http://127.0.0.1:3000/books -H 'content-type: application/json' -d '{"title":"Dune"}'
curl -i -X POST http://127.0.0.1:3000/books -H 'content-type: application/json' -d '{"title":"Emma"}'
curl -i http://127.0.0.1:3000/books/count
