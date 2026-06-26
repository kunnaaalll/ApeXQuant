terraform {
  required_version = ">= 1.5.0"
  required_providers {
    aws = {
      source  = "hashicorp/aws"
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
  }
}

provider "aws" {
  region = var.aws_region
}

# --- VPC Module ---
module "vpc" {
  source  = "terraform-aws-modules/vpc/aws"
  version = "5.1.2"

  name = "apex-vpc"
  cidr = var.vpc_cidr

  azs             = ["${var.aws_region}a", "${var.aws_region}b", "${var.aws_region}c"]
  private_subnets = var.private_subnets
  public_subnets  = var.public_subnets

  enable_nat_gateway = true
  single_nat_gateway = false
  enable_vpn_gateway = false

  tags = {
    Environment = var.environment
    Project     = "APEX V3"
  }
}

# --- EKS Module ---
module "eks" {
  source  = "terraform-aws-modules/eks/aws"
  version = "19.16.0"

  cluster_name    = "apex-cluster-${var.environment}"
  cluster_version = "1.28"

  vpc_id                         = module.vpc.vpc_id
  subnet_ids                     = module.vpc.private_subnets
  cluster_endpoint_public_access = true

  eks_managed_node_groups = {
    general = {
      desired_size = 3
      min_size     = 3
      max_size     = 10

      instance_types = ["t3.xlarge"]
      capacity_type  = "ON_DEMAND"
    }
  }

  tags = {
    Environment = var.environment
    Project     = "APEX V3"
  }
}

# --- RDS PostgreSQL HA Module ---
module "db" {
  source  = "terraform-aws-modules/rds/aws"
  version = "6.1.1"

  identifier = "apex-postgres-${var.environment}"

  engine               = "postgres"
  engine_version       = "14"
  family               = "postgres14"
  major_engine_version = "14"
  instance_class       = "db.t4g.large"

  allocated_storage     = 100
  max_allocated_storage = 500

  db_name  = "apex_v3"
  username = "apex"
  port     = 5432

  multi_az               = true
  db_subnet_group_name   = module.vpc.database_subnet_group_name
  vpc_security_group_ids = [module.rds_sg.security_group_id]

  maintenance_window      = "Mon:00:00-Mon:03:00"
  backup_window           = "03:00-06:00"
  enabled_cloudwatch_logs_exports = ["postgresql", "upgrade"]
  create_cloudwatch_log_group     = true

  backup_retention_period = 14
  skip_final_snapshot     = false
  deletion_protection     = true
}

module "rds_sg" {
  source  = "terraform-aws-modules/security-group/aws"
  version = "~> 5.0"

  name        = "apex-rds-sg"
  description = "Security group for RDS"
  vpc_id      = module.vpc.vpc_id

  ingress_with_cidr_blocks = [
    {
      from_port   = 5432
      to_port     = 5432
      protocol    = "tcp"
      description = "PostgreSQL access from within VPC"
      cidr_blocks = module.vpc.vpc_cidr_block
    },
  ]
}

# --- ElastiCache Redis Cluster (Event Bus) ---
resource "aws_elasticache_subnet_group" "apex" {
  name       = "apex-redis-subnet-group"
  subnet_ids = module.vpc.private_subnets
}

resource "aws_elasticache_replication_group" "apex_redis" {
  replication_group_id          = "apex-redis-cluster"
  description                   = "APEX Redis Cluster for Event Bus"
  node_type                     = "cache.m6g.large"
  port                          = 6379
  parameter_group_name          = "default.redis7.cluster.on"
  automatic_failover_enabled    = true
  subnet_group_name             = aws_elasticache_subnet_group.apex.name
  security_group_ids            = [module.redis_sg.security_group_id]
  
  num_node_groups         = 2
  replicas_per_node_group = 1
}

module "redis_sg" {
  source  = "terraform-aws-modules/security-group/aws"
  version = "~> 5.0"

  name        = "apex-redis-sg"
  description = "Security group for Redis"
  vpc_id      = module.vpc.vpc_id

  ingress_with_cidr_blocks = [
    {
      from_port   = 6379
      to_port     = 6379
      protocol    = "tcp"
      description = "Redis access from within VPC"
      cidr_blocks = module.vpc.vpc_cidr_block
    },
  ]
}

# --- AWS Secrets Manager ---
resource "aws_secretsmanager_secret" "apex_secrets" {
  name = "apex/production/secrets"
  recovery_window_in_days = 7
}

resource "aws_secretsmanager_secret_version" "apex_secrets_version" {
  secret_id     = aws_secretsmanager_secret.apex_secrets.id
  secret_string = jsonencode({
    databaseUrl = "postgres://apex:${module.db.db_instance_password}@${module.db.db_instance_endpoint}/apex_v3"
    redisUrl    = "redis://${aws_elasticache_replication_group.apex_redis.configuration_endpoint_address}:6379"
  })
}
