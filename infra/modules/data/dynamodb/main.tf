resource "aws_dynamodb_table" "guilds" {
  name           = "kb2_guilds_${var.deployment_env}"
  billing_mode   = "PROVISIONED"
  read_capacity  = 1
  write_capacity = 1
  hash_key       = "guild_id"

  attribute {
    name = "guild_id"
    type = "S"
  }

  tags = {
    Name        = "kb2_guilds_${var.deployment_env}"
    Environment = var.deployment_env
  }
}

resource "aws_dynamodb_table" "users" {
  name           = "kb2_users_${var.deployment_env}"
  billing_mode   = "PROVISIONED"
  read_capacity  = 1
  write_capacity = 1
  hash_key       = "user_id"

  attribute {
    name = "user_id"
    type = "S"
  }

  tags = {
    Name        = "kb2_users_${var.deployment_env}"
    Environment = var.deployment_env
  }
}