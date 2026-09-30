# serve: -p middleware-and-tower-layers --example mw-cors-origin
curl -i -H 'origin: https://myapp.com' http://127.0.0.1:3000/
curl -i -H 'origin: https://example.com' http://127.0.0.1:3000/
