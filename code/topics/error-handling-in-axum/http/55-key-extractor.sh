# serve: -p error-handling-in-axum --example errors-key-extractor
curl -i http://127.0.0.1:3000/secret
curl -i -H 'x-api-key: guess' http://127.0.0.1:3000/secret
curl -i -H 'x-api-key: letmein' http://127.0.0.1:3000/secret
