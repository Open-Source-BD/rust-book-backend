# serve: -p path-and-query-extractors --example extractors-range
curl -i "http://127.0.0.1:3000/range?from=1&to=5"
curl -i "http://127.0.0.1:3000/range?from=5&to=1"
curl -i "http://127.0.0.1:3000/range?from=1"
