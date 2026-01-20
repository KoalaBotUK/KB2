data "aws_region" "current" {}
data "aws_caller_identity" "current" {}

data "aws_iam_policy_document" "assume_role" {
  statement {
    effect = "Allow"

    principals {
      type = "Service"
      identifiers = ["lambda.amazonaws.com"]
    }

    actions = ["sts:AssumeRole"]
  }
}

resource "aws_iam_role" "lambda_execute_role" {
  name               = "kb2-lambda-execute-role-${var.deployment_env}"
  assume_role_policy = data.aws_iam_policy_document.assume_role.json
}

resource "aws_iam_role_policy_attachment" "dynamodb_role_attach" {
  role       = aws_iam_role.lambda_execute_role.name
  policy_arn = "arn:aws:iam::aws:policy/AmazonDynamoDBFullAccess_v2"
}

resource "aws_iam_role_policy_attachment" "ses_role_attach" {
  role       = aws_iam_role.lambda_execute_role.name
  policy_arn = "arn:aws:iam::aws:policy/AmazonSESFullAccess"
}

resource "aws_cloudwatch_log_group" "default" {
  name              = "/aws/lambda/${aws_lambda_function.lambda_function.function_name}" # Replace with your log group name
  retention_in_days = 14                              # Set the desired retention period in days
}

data "aws_iam_policy_document" "cloudwatch_readwrite" {
  statement {
    effect = "Allow"
    actions = ["logs:CreateLogGroup",]
    resources = ["arn:aws:logs:${data.aws_region.current.region}:${data.aws_caller_identity.current.account_id}:*"]
  }

  statement {
    effect = "Allow"
    actions = ["logs:CreateLogStream", "logs:PutLogEvents",]
    resources = [
      "${aws_cloudwatch_log_group.default.arn}:*"
    ]
  }
}

resource "aws_iam_policy" "cloudwatch_readwrite" {
  name   = "kb2-cloudwatch-readwrite-policy-${var.deployment_env}"
  policy = data.aws_iam_policy_document.cloudwatch_readwrite.json
}


resource "aws_iam_role_policy_attachment" "execution_role_attach" {
  role       = aws_iam_role.lambda_execute_role.name
  policy_arn = aws_iam_policy.cloudwatch_readwrite.arn
}

data "aws_iam_policy_document" "dsql_dbconnect" {
  statement {
    effect = "Allow"
    actions = ["dsql:DbConnectAdmin",]
    resources = [var.dsql_arn]
  }
}

resource "aws_iam_policy" "dsql_dbconnect" {
  name   = "kb2-dsql-dbconnect-policy-${var.deployment_env}"
  policy = data.aws_iam_policy_document.dsql_dbconnect.json
}

resource "aws_iam_role_policy_attachment" "dsql_dbconnect_attach" {
  role       = aws_iam_role.lambda_execute_role.name
  policy_arn = aws_iam_policy.dsql_dbconnect.arn
}

data "archive_file" "empty_zip" {
  type        = "zip"
  output_path = "${path.module}/bootstrap.zip"

  source {
    content  = "kb2"
    filename = "bootstrap"
  }
}

resource "aws_lambda_function" "lambda_function" {
  function_name = "kb2-lambda-function-${var.deployment_env}"
  role          = aws_iam_role.lambda_execute_role.arn
  handler       = "main"
  filename      = data.archive_file.empty_zip.output_path
  timeout = 10

  runtime = "provided.al2023"

  environment {
    variables = {
      AWS_LAMBDA_HTTP_IGNORE_STAGE_IN_PATH = "true"
      AWS_LAMBDA_LOG_LEVEL                 = "info"
      API_GATEWAY_BASE_PATH                = "/default"
      DEPLOYMENT_ENV                       = var.deployment_env
      DISCORD_BOT_TOKEN                    = var.discord_bot_token
      DISCORD_PUBLIC_KEY                   = var.discord_public_key
      DSQL_USER                            = var.dsql_user
      DSQL_ENDPOINT                        = var.dsql_endpoint
    }
  }
}