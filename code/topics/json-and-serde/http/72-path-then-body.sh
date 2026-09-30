# serve: -p json-and-serde --example json-path-then-body
curl -i -X PUT http://127.0.0.1:3000/books/7 -H 'content-type: application/json' -d '{"title":"Emma","in_stock":true}'
