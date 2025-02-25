locals {
  postgres_password = can(coalesce(trim(var.postgres_password, " "))) ? var.postgres_password : random_string.postgres_random_password.result
}

resource "random_string" "postgres_random_password" {
  length = 16
}
