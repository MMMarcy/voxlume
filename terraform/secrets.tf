locals {
  postgres_password = can(coalesce(trim(var.postgres_password, " "))) ? var.postgres_password : random_string.postgres_random_password.result
  neo4j_password    = can(coalesce(trim(var.neo4j_password, " "))) ? var.neo4j_password : random_string.neo4j_random_password.result
  pgmq_password     = can(coalesce(trim(var.pgmq_password, " "))) ? var.pgmq_password : random_string.pgmq_random_password.result
}

resource "random_string" "postgres_random_password" {
  length = 16
}

resource "random_string" "neo4j_random_password" {
  length = 16
}

resource "random_string" "pgmq_random_password" {
  length = 16
}
