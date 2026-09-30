# serve: -p middleware-and-tower-layers
curl -i -H 'origin: https://example.com' http://127.0.0.1:3000/
curl -i -X OPTIONS -H 'origin: https://example.com' -H 'access-control-request-method: DELETE' http://127.0.0.1:3000/
