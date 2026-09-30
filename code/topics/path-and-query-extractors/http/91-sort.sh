# serve: -p path-and-query-extractors --example extractors-sort
curl -i "http://127.0.0.1:3000/search?q=rust"
curl -i "http://127.0.0.1:3000/search?q=rust&sort=year"
