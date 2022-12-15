
variable "digital_ocean_api_token" {
  type    = string
  default = "${env("DO_API_TOKEN")}"
}

source "digitalocean" "do" {
  api_token     = "${var.digital_ocean_api_token}"
  image         = "centos-stream-9-x64"
  region        = "nyc3"
  size          = "c-2"
  snapshot_name = "logsnarf-${timestamp()}"
  ssh_username  = "root"
}

source "docker" "docker" {
  image = "centos:stream9"
  commit = "true"
}

build {
  sources = ["source.digitalocean.do", "source.docker.docker"]

  provisioner "file" {
    destination = "/tmp"
    source      = "ops/templates"
  }

  provisioner "file" {
    destination = "/tmp"
    source      = "ops/environments"
  }

  provisioner "shell" {
    environment_vars = ["RACK_ENV=production"]
    scripts          = ["ops/scripts/logsnarf.sh"]
  }
}
