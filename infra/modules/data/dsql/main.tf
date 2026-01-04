resource "aws_dsql_cluster" "default" {
  deletion_protection_enabled = true

  tags = {
    Name = "kb2_${var.deployment_env}"
  }
}
