# serve: -p input-validation --example validate-website
curl -i -X POST http://127.0.0.1:3000/sign-up -H 'content-type: application/json' -d '{"username":"ada","email":"ada@example.com","age":36}'
curl -i -X POST http://127.0.0.1:3000/sign-up -H 'content-type: application/json' -d '{"username":"ada","email":"ada@example.com","age":36,"website":"my site"}'
curl -i -X POST http://127.0.0.1:3000/sign-up -H 'content-type: application/json' -d '{"username":"ada","email":"ada@example.com","age":36,"website":"https://example.com"}'
