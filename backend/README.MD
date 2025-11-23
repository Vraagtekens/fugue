```
    sea-orm-cli generate entity \
    -u postgres://postgres:postgres@localhost:5433/fugue \
    -o src/entities
```

[alias]
generate-entities = "sea-orm-cli generate entity --database-url ${DATABASE_URL} --output-dir src/entities --with-serde both --model-extra-attributes 'ts(export)' --frontend-format"

sea-orm-cli generate entity \
 -o src/entities
--with-serde
