# serve: -p middleware-and-tower-layers --example mw-require-key
curl -i http://127.0.0.1:3000/
curl -i -H 'x-api-key: guess' http://127.0.0.1:3000/
curl -i -H 'x-api-key: letmein' http://127.0.0.1:3000/
curl -i http://127.0.0.1:3000/missing
