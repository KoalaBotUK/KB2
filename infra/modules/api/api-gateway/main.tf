data "aws_region" "current" {}
data "aws_caller_identity" "current" {}

resource "aws_api_gateway_rest_api" "default" {
  name = "kb2-api-${var.deployment_env}"

  binary_media_types = ["*/*"]
}

resource "aws_api_gateway_resource" "lambda_proxy" {
  rest_api_id = aws_api_gateway_rest_api.default.id
  parent_id   = aws_api_gateway_rest_api.default.root_resource_id
  path_part   = "{proxy+}"
}

resource "aws_api_gateway_method" "lambda_proxy" {
  rest_api_id   = aws_api_gateway_rest_api.default.id
  resource_id   = aws_api_gateway_resource.lambda_proxy.id
  authorization = "NONE"
  http_method   = "ANY"
}

resource "aws_api_gateway_method_response" "lambda_proxy_response_200" {
  rest_api_id = aws_api_gateway_rest_api.default.id
  resource_id = aws_api_gateway_resource.lambda_proxy.id
  http_method = aws_api_gateway_method.lambda_proxy.http_method
  status_code = "200"
}

resource "aws_api_gateway_integration" "lambda_proxy" {
  rest_api_id             = aws_api_gateway_rest_api.default.id
  resource_id             = aws_api_gateway_resource.lambda_proxy.id
  http_method             = aws_api_gateway_method.lambda_proxy.http_method
  integration_http_method = "POST"
  type                    = "AWS_PROXY"
  uri                     = var.lambda_function_invoke_arn
}

resource "aws_lambda_permission" "apigw_lambda" {
  statement_id  = "AllowExecutionFromAPIGateway"
  action        = "lambda:InvokeFunction"
  function_name = var.lambda_function_name
  principal     = "apigateway.amazonaws.com"

  source_arn = "arn:aws:execute-api:${data.aws_region.current.name}:${data.aws_caller_identity.current.account_id}:${aws_api_gateway_rest_api.default.id}/*/*/*"
}

resource "aws_api_gateway_deployment" "default" {
  rest_api_id = aws_api_gateway_rest_api.default.id

  triggers = {
    # NOTE: The configuration below will satisfy ordering considerations,
    #       but not pick up all future REST API changes. More advanced patterns
    #       are possible, such as using the filesha1() function against the
    #       Terraform configuration file(s) or removing the .id references to
    #       calculate a hash against whole resources. Be aware that using whole
    #       resources will show a difference after the initial implementation.
    #       It will stabilize to only change when resources change afterwards.
    redeployment = sha1(jsonencode([
      aws_api_gateway_resource.lambda_proxy,
      aws_api_gateway_method.lambda_proxy,
      aws_api_gateway_method_response.lambda_proxy_response_200,
      aws_api_gateway_integration.lambda_proxy,
    ]))
  }

  lifecycle {
    create_before_destroy = true
  }
}

resource "aws_api_gateway_stage" "default" {
  rest_api_id   = aws_api_gateway_rest_api.default.id
  deployment_id = aws_api_gateway_deployment.default.id
  stage_name    = "default"
}

resource "aws_api_gateway_base_path_mapping" "default" {
  api_id      = aws_api_gateway_rest_api.default.id
  stage_name  = aws_api_gateway_stage.default.stage_name
  domain_name = "api.${var.deployment_env}.${var.root_domain_name}" # TODO: Create Domain Name Resource
}

resource "aws_api_gateway_usage_plan" "default" {
  name         = "kb2-usage-plan-${var.deployment_env}"
  description  = "Usage Plan for Rate Limiting and Throttling"

  api_stages {
    api_id = aws_api_gateway_rest_api.default.id
    stage  = aws_api_gateway_stage.default.stage_name
  }

  throttle_settings {
    burst_limit = 10
    rate_limit  = 5
  }
}