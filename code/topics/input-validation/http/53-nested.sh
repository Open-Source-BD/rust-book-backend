# serve: -p input-validation --example validate-nested
curl -i -X POST http://127.0.0.1:3000/sign-up -H 'content-type: application/json' -d '{"username":"ada","address":{"city":"Dhaka","postcode":"1207"}}'
curl -i -X POST http://127.0.0.1:3000/sign-up -H 'content-type: application/json' -d '{"username":"al","address":{"city":"","postcode":"12"}}'
