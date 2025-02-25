locals {
  postgres_port = 5432
}

resource "docker_container" "postgres_container" {
  name = "postgres_container"

  image = docker_image.postgres_docker_image.image_id
  env = toset([
    "POSTGRES_PASSWORD=${local.postgres_password}"]
  )
  mounts {
    target = "/usr/local/psql/data"
    source = "${abspath(path.root)}/../docker_volumes/postgres"
    type   = "bind"
  }
  restart      = "unless-stopped"
  network_mode = "host"
  ports {
    internal = local.postgres_port
    external = local.postgres_port
    protocol = "TCP"
  }
  lifecycle {
    replace_triggered_by = [
      docker_image.postgres_docker_image.image_id
    ]
  }
}

