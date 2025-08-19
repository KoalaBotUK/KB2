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

  lifecycle {
    ignore_changes = [read_capacity, write_capacity]
  }
}

resource "aws_appautoscaling_target" "guild_read_target" {
  max_capacity       = 10
  min_capacity       = 1
  resource_id        = "table/${aws_dynamodb_table.guilds.name}"
  scalable_dimension = "dynamodb:table:ReadCapacityUnits"
  service_namespace  = "dynamodb"
}

resource "aws_appautoscaling_policy" "guilds_read_policy" {
  name               = "DynamoDBReadCapacityUtilization:${aws_appautoscaling_target.guild_read_target.resource_id}"
  policy_type        = "TargetTrackingScaling"
  resource_id        = aws_appautoscaling_target.guild_read_target.resource_id
  scalable_dimension = aws_appautoscaling_target.guild_read_target.scalable_dimension
  service_namespace  = aws_appautoscaling_target.guild_read_target.service_namespace

  target_tracking_scaling_policy_configuration {
    predefined_metric_specification {
      predefined_metric_type = "DynamoDBReadCapacityUtilization"
    }

    target_value = 70.0
  }
}

resource "aws_appautoscaling_target" "guilds_write_target" {
  max_capacity       = 5
  min_capacity       = 1
  resource_id        = "table/${aws_dynamodb_table.guilds.name}"
  scalable_dimension = "dynamodb:table:WriteCapacityUnits"
  service_namespace  = "dynamodb"
}

resource "aws_appautoscaling_policy" "guilds_write_policy" {
  name               = "DynamoDBWriteCapacityUtilization:${aws_appautoscaling_target.guilds_write_target.resource_id}"
  policy_type        = "TargetTrackingScaling"
  resource_id        = aws_appautoscaling_target.guilds_write_target.resource_id
  scalable_dimension = aws_appautoscaling_target.guilds_write_target.scalable_dimension
  service_namespace  = aws_appautoscaling_target.guilds_write_target.service_namespace

  target_tracking_scaling_policy_configuration {
    predefined_metric_specification {
      predefined_metric_type = "DynamoDBWriteCapacityUtilization"
    }

    target_value = 70.0
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

  lifecycle {
    ignore_changes = [read_capacity, write_capacity]
  }
}

resource "aws_appautoscaling_target" "users_read_target" {
  max_capacity       = 10
  min_capacity       = 1
  resource_id        = "table/${aws_dynamodb_table.users.name}"
  scalable_dimension = "dynamodb:table:ReadCapacityUnits"
  service_namespace  = "dynamodb"
}

resource "aws_appautoscaling_policy" "users_read_policy" {
  name               = "DynamoDBReadCapacityUtilization:${aws_appautoscaling_target.users_read_target.resource_id}"
  policy_type        = "TargetTrackingScaling"
  resource_id        = aws_appautoscaling_target.users_read_target.resource_id
  scalable_dimension = aws_appautoscaling_target.users_read_target.scalable_dimension
  service_namespace  = aws_appautoscaling_target.users_read_target.service_namespace

  target_tracking_scaling_policy_configuration {
    predefined_metric_specification {
      predefined_metric_type = "DynamoDBReadCapacityUtilization"
    }

    target_value = 70.0
  }
}

resource "aws_appautoscaling_target" "users_write_target" {
  max_capacity       = 5
  min_capacity       = 1
  resource_id        = "table/${aws_dynamodb_table.users.name}"
  scalable_dimension = "dynamodb:table:WriteCapacityUnits"
  service_namespace  = "dynamodb"
}

resource "aws_appautoscaling_policy" "users_write_policy" {
  name               = "DynamoDBWriteCapacityUtilization:${aws_appautoscaling_target.users_write_target.resource_id}"
  policy_type        = "TargetTrackingScaling"
  resource_id        = aws_appautoscaling_target.users_write_target.resource_id
  scalable_dimension = aws_appautoscaling_target.users_write_target.scalable_dimension
  service_namespace  = aws_appautoscaling_target.users_write_target.service_namespace

  target_tracking_scaling_policy_configuration {
    predefined_metric_specification {
      predefined_metric_type = "DynamoDBWriteCapacityUtilization"
    }

    target_value = 70.0
  }
}

