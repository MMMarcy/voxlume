locals {
  postgres_password = can(coalesce(trim(var.postgres_password, " "))) ? var.postgres_password : random_string.postgres_random_password.result
  neo4j_password    = can(coalesce(trim(var.neo4j_password, " "))) ? var.neo4j_password : random_string.neo4j_random_password.result
  redis_password    = can(coalesce(trim(var.redis_password, " "))) ? var.redis_password : random_string.redis_random_password.result
}

resource "random_string" "postgres_random_password" {
  length = 16
}

resource "random_string" "neo4j_random_password" {
  length = 16
}

resource "random_string" "redis_random_password" {
  length = 16
}
