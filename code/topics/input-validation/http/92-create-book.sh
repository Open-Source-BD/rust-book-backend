# serve: -p input-validation --example validate-create-book
curl -i -X POST http://127.0.0.1:3000/books -H 'content-type: application/json' -d '{"title":"Dune","pages":412,"isbn":"9780441013593"}'
curl -i -X POST http://127.0.0.1:3000/books -H 'content-type: application/json' -d '{"title":"","pages":0,"isbn":"978-0441013593"}'
