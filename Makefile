post-database.db: db-init db-apply

test:
	cargo test

db-plan:
	atlas schema diff \
		--dev-url "sqlite://post-database.db" \
		--from "sqlite://post-database.db" \
		--to "file://db/schema.hcl"

db-init:
	sqlite3 -batch "post-database.db" ""

db-apply:
	atlas schema apply --auto-approve \
		--url "sqlite://post-database.db" \
		--to "file://db/schema.hcl"

