data "docker_registry_image" "postgres_docker_image" {
  name = "${var.postgres_docker_image_data.image_name}:${var.postgres_docker_image_data.tag}"
}

resource "docker_image" "postgres_docker_image" {
  name          = data.docker_registry_image.postgres_docker_image.name
  keep_locally  = true
  pull_triggers = [data.docker_registry_image.postgres_docker_image.sha256_digest]
}
