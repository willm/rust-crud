schema "main" {}

table "posts" {
  schema = schema.main

  column "id" {
    null = false
    type = integer
    auto_increment = true
  }

  column "body" {
    type = varchar
    null = false
  }

  column "author" {
    type = varchar(50)
    null = false
  }

  primary_key {
    columns = [
      column.id
    ]
  }
}
