variable "postgres_password" {
  type        = string
  description = "Postgres password. If empty, a random one will be generated."
  default     = "password"
}

variable "neo4j_password" {
  type        = string
  description = "Neo4j password. If empty, a random one will be generated."
  default     = "password"
}

variable "postgres_docker_image_data" {
  type = object({
    image_name = string
    tag        = string
  })
  description = "Object with the information about the pgmq docker image."
  default = {
    image_name = "postgres"
    tag        = "17"
  }
}

variable "neo4j_docker_image_data" {
  type = object({
    image_name = string
    tag        = string
  })
  description = "Object with the information about the neo4j docker image."
  default = {
    image_name = "neo4j"
    tag        = "2025.02.0"
  }
}
