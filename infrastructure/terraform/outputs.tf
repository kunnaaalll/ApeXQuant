output "gke_cluster_name" {
  description = "The name of the GKE cluster"
  value       = google_container_cluster.apex_cluster.name
}

output "gke_cluster_endpoint" {
  description = "Endpoint for GKE control plane"
  value       = google_container_cluster.apex_cluster.endpoint
}

output "cloud_sql_connection_name" {
  description = "Cloud SQL connection name"
  value       = google_sql_database_instance.apex_postgres.connection_name
}

output "redis_host" {
  description = "Redis host"
  value       = google_redis_instance.apex_redis.host
}

output "secret_id" {
  description = "ID of the Secret Manager secret"
  value       = google_secret_manager_secret.apex_secrets.secret_id
}
