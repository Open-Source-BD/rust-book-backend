SET client_min_messages = warning;
DROP TABLE IF EXISTS pets;
CREATE TABLE pets (
    name text,
    kind text
);
INSERT INTO pets (name, kind) VALUES ('Rex', 'dog'), ('Tom', 'cat'), ('Nemo', 'fish');
SELECT * FROM pets;
