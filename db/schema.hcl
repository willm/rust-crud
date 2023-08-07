schema "main" {}

table "users" {
  schema = schema.main

  column "id" {
    null = false
    type = integer
    auto_increment = true
  }

  column "email" {
    null = false
    type = varchar(100)
  }

  primary_key {
    columns = [column.id]
  }
}

table "user_credential_challenges" {
  schema = schema.main

  column "user_id" {
    null = false
    type = integer
  }

  column "challenge" {
    null = false
    type = varchar(50)
  }

  foreign_key "user_id" {
    columns = [column.user_id]
    ref_columns = [table.users.column.id]
  }
}

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
