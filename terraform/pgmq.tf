locals {
  pgmq_port = 5433
}


resource "docker_container" "pgmq_container" {
  name = "pgmq_container"

  image = docker_image.pgmq_docker_image.image_id
  env = toset([
    "POSTGRES_DATABASE=voxlume",
    "POSTGRES_USER=postgres",
    "POSTGRES_PASSWORD=${local.pgmq_password}",
    ]
  )
  user = "root"
  mounts {
    target = "/var/lib/postgresql/data"
    source = "${abspath(path.root)}/../docker_volumes/pgmq"
    type   = "bind"
  }
  restart      = "always"
  network_mode = "host"
  command      = ["-p", local.pgmq_port]
  ports {
    internal = local.pgmq_port
    external = local.pgmq_port
    protocol = "TCP"
  }
  lifecycle {
    replace_triggered_by = [
      docker_image.pgmq_docker_image.image_id
    ]
  }
}

