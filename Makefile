include ./server/.env
export

# HELP =================================================================================================================
# This will output the help for each task
# thanks to https://marmelab.com/blog/2016/02/29/auto-documented-makefile.html
.PHONY: help

help: ## Display this help screen
	@awk 'BEGIN {FS = ":.*##"; printf "\nUsage:\n  make \033[36m<target>\033[0m\n"} /^[a-zA-Z_-]+:.*?##/ { printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2 } /^##@/ { printf "\n\033[1m%s\033[0m\n", substr($$0, 5) } ' $(MAKEFILE_LIST)

compose-up: ### Run docker-compose
	docker-compose up && docker-compose logs -f
.PHONY: compose-up

createdb: ### Create chat database
	DATABASE_URL=${DATABASE_URL} sqlx database create
.PHONY: createdb

dropdb: ### Drop chat database
	DATABASE_URL=${DATABASE_URL} sqlx database drop
.PHONY: dropdb

sqlx: ### Install sqlx
	cargo install sqlx-cli --no-default-features --features native-tls,postgres
.PHONY: sqlx

migration: ### Add migration
	sqlx migrate add -r chat --source ./server/migrations
.PHONY: migration

migrateup: ### Migration up
	DATABASE_URL=${DATABASE_URL} sqlx migrate run --source ./server/migrations
.PHONY: migrateup

migratedown: ### Migration down
	DATABASE_URL=${DATABASE_URL} sqlx migrate revert --source ./server/migrations
.PHONY: migratedown

prepare: ### Prepare sqlx offline
	cargo sqlx prepare
.PHONY: prepare

server: ### Run server
	cargo run
.PHONY: server

watch-css:
	npx tailwindcss -i ./client/input.css -o ./client/public/tailwind.css --watch 

watch-client: ### Run clent watch	
	dx serve --bin client --hot-reload
.PHONY: client-dev

client-dev: watch-client
