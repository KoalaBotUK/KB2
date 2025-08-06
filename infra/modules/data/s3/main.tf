resource "aws_s3_bucket" "api_bucket" {
    bucket = "kb2-api-${var.deployment_env}"

  tags = {
    Name        = "kb2-api-${var.deployment_env}"
    Environment = var.deployment_env
  }
}
