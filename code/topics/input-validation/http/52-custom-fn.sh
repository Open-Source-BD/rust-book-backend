# serve: -p input-validation --example validate-custom-fn
curl -i -X POST http://127.0.0.1:3000/sign-up -H 'content-type: application/json' -d '{"username":"ada lovelace","email":"ada@example.com","age":36}'
curl -i -X POST http://127.0.0.1:3000/sign-up -H 'content-type: application/json' -d '{"username":"ada","email":"ada@example.com","age":36}'
curl -i -X POST http://127.0.0.1:3000/sign-up -H 'content-type: application/json' -d '{"username":"ada king, countess of lovelace","email":"ada@example.com","age":36}'
