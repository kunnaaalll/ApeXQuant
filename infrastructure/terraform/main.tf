terraform {
  required_version = ">= 1.5.0"
  required_providers {
    google = {
      source  = "hashicorp/google"
      version = "~> 5.0"
    }
    kubernetes = {
      source  = "hashicorp/kubernetes"
      version = "~> 2.23"
    }
    helm = {
      source  = "hashicorp/helm"
      version = "~> 2.11"
    }
    random = {
      source  = "hashicorp/random"
      version = "~> 3.5"
    }
  }
}

provider "google" {
  project = var.gcp_project_id
  region  = var.gcp_region
}

# --- VPC & Networking ---
resource "google_compute_network" "apex_vpc" {
  name                    = "apex-vpc-${var.environment}"
  auto_create_subnetworks = false
}

resource "google_compute_subnetwork" "apex_subnet" {
  name                     = "apex-subnet-${var.environment}"
  ip_cidr_range            = "10.0.0.0/16"
  region                   = var.gcp_region
  network                  = google_compute_network.apex_vpc.id
  private_ip_google_access = true
}

resource "google_compute_router" "router" {
  name    = "apex-router-${var.environment}"
  region  = google_compute_subnetwork.apex_subnet.region
  network = google_compute_network.apex_vpc.id
}

resource "google_compute_router_nat" "nat" {
  name                               = "apex-nat-${var.environment}"
  router                             = google_compute_router.router.name
  region                             = google_compute_router.router.region
  nat_ip_allocate_option             = "AUTO_ONLY"
  source_subnetwork_ip_ranges_to_nat = "ALL_SUBNETWORKS_ALL_IP_RANGES"
}

# Private Service Access for Cloud SQL
resource "google_compute_global_address" "private_ip_address" {
  name          = "apex-private-ip-${var.environment}"
  purpose       = "VPC_PEERING"
  address_type  = "INTERNAL"
  prefix_length = 16
  network       = google_compute_network.apex_vpc.id
}

resource "google_service_networking_connection" "private_vpc_connection" {
  network                 = google_compute_network.apex_vpc.id
  service                 = "servicenetworking.googleapis.com"
  reserved_peering_ranges = [google_compute_global_address.private_ip_address.name]
}

# --- GKE Autopilot Cluster ---
resource "google_container_cluster" "apex_cluster" {
  name       = "apex-cluster-${var.environment}"
  location   = var.gcp_region
  network    = google_compute_network.apex_vpc.id
  subnetwork = google_compute_subnetwork.apex_subnet.id
  
  enable_autopilot = true

  private_cluster_config {
    enable_private_nodes    = true
    enable_private_endpoint = false
    master_ipv4_cidr_block  = "172.16.0.0/28"
  }
}

# --- Cloud SQL PostgreSQL HA ---
resource "random_password" "db_password" {
  length  = 16
  special = false
}

resource "google_sql_database_instance" "apex_postgres" {
  name             = "apex-postgres-${var.environment}"
  database_version = "POSTGRES_14"
  region           = var.gcp_region
  depends_on       = [google_service_networking_connection.private_vpc_connection]

  settings {
    tier              = "db-custom-2-7680" # Custom machine type equivalent to t4g.large
    availability_type = "REGIONAL"         # High Availability
    disk_size         = 100
    disk_autoresize   = true

    ip_configuration {
      ipv4_enabled    = false
      private_network = google_compute_network.apex_vpc.id
    }
    
    backup_configuration {
      enabled    = true
      start_time = "03:00"
    }
  }
}

resource "google_sql_user" "apex_user" {
  name     = "apex"
  instance = google_sql_database_instance.apex_postgres.name
  password = random_password.db_password.result
}

resource "google_sql_database" "apex_db" {
  name     = "apex_v3"
  instance = google_sql_database_instance.apex_postgres.name
}

# --- Memorystore Redis (Event Bus) ---
resource "google_redis_instance" "apex_redis" {
  name               = "apex-redis-${var.environment}"
  tier               = "STANDARD_HA"
  memory_size_gb     = 5
  region             = var.gcp_region
  authorized_network = google_compute_network.apex_vpc.id
  redis_version      = "REDIS_7_0"
}

# --- Secret Manager ---
resource "google_secret_manager_secret" "apex_secrets" {
  secret_id = "apex-production-secrets"

  replication {
    auto {}
  }
}

resource "google_secret_manager_secret_version" "apex_secrets_version" {
  secret      = google_secret_manager_secret.apex_secrets.id
  secret_data = jsonencode({
    databaseUrl = "postgres://${google_sql_user.apex_user.name}:${random_password.db_password.result}@${google_sql_database_instance.apex_postgres.private_ip_address}/apex_v3"
    redisUrl    = "redis://${google_redis_instance.apex_redis.host}:${google_redis_instance.apex_redis.port}"
  })
}
