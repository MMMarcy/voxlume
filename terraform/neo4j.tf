locals {
  neo4j_ports = [7474, 7687]
}

resource "docker_container" "neo4j_container" {
  name = "neo4j_container"

  image = docker_image.neo4j_docker_image.image_id
  env = toset([
    "NEO4J_AUTH=neo4j/${local.neo4j_password}"]
  )
  mounts {
    target = "/data"
    source = "${abspath(path.root)}/../docker_volumes/neo4j"
    type   = "bind"
  }
  restart      = "unless-stopped"
  network_mode = "host"
  ports {
    internal = local.neo4j_ports[0]
    external = local.neo4j_ports[0]
    protocol = "TCP"
  }
  ports {
    internal = local.neo4j_ports[1]
    external = local.neo4j_ports[1]
    protocol = "TCP"
  }

  lifecycle {
    replace_triggered_by = [
      docker_image.neo4j_docker_image.image_id
    ]
  }
}

