# serve: -p middleware-and-tower-layers --example mw-route-layer
curl -i http://127.0.0.1:3000/
curl -i http://127.0.0.1:3000/admin
curl -i -H 'x-api-key: letmein' http://127.0.0.1:3000/admin
curl -i http://127.0.0.1:3000/missing
