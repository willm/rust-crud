schema "main" {}

table "posts" {
  schema = schema.main

  column "id" {
    type = int
  }

  column "body" {
    type = varchar
  }

  column "author" {
    type = varchar(50)
  }

  primary_key {
    columns = [
      column.id
    ]
  }
}
