data "aws_region" "current" {}
data "aws_caller_identity" "current" {}

resource "aws_api_gateway_rest_api" "api" {
  name = "kb2-api-${var.deployment_env}"
}

resource "aws_api_gateway_resource" "default" {
  rest_api_id = aws_api_gateway_rest_api.api.id
  parent_id   = aws_api_gateway_rest_api.api.root_resource_id
  path_part   = "{proxy+}"
}

resource "aws_api_gateway_method" "get_s3_default" {
  rest_api_id   = aws_api_gateway_rest_api.api.id
  resource_id   = aws_api_gateway_resource.default.id
  authorization = "NONE"
  http_method   = "GET"

  request_parameters = {
    "method.request.path.proxy" = true
  }
}

resource "aws_api_gateway_resource" "api" {
  rest_api_id = aws_api_gateway_rest_api.api.id
  parent_id   = aws_api_gateway_rest_api.api.root_resource_id
  path_part   = "api"
}

resource "aws_api_gateway_resource" "lambda_proxy" {
  for_each = {
    for key, value in [aws_api_gateway_resource.api] : value.path_part => value.id
  }
  rest_api_id = aws_api_gateway_rest_api.api.id
  parent_id   = each.value
  path_part   = "{proxy+}"
}


resource "aws_api_gateway_method" "lambda_proxy" {
  for_each      = aws_api_gateway_resource.lambda_proxy
  rest_api_id   = aws_api_gateway_rest_api.api.id
  resource_id   = each.value.id
  authorization = "NONE"
  http_method   = "ANY"
}

resource "aws_api_gateway_method_response" "lambda_proxy_response_200" {
  for_each    = aws_api_gateway_resource.lambda_proxy
  rest_api_id = aws_api_gateway_rest_api.api.id
  resource_id = each.value.id
  http_method = "ANY"
  status_code = "200"

  response_models = {
    "application/json" = "Empty"
  }
}

resource "aws_api_gateway_integration" "lambda_proxy" {
  depends_on = [aws_api_gateway_method.lambda_proxy]
  for_each                = aws_api_gateway_resource.lambda_proxy
  rest_api_id             = aws_api_gateway_rest_api.api.id
  resource_id             = each.value.id
  http_method             = "ANY"
  integration_http_method = "POST"
  type                    = "AWS_PROXY"
  uri                     = var.lambda_function_invoke_arn
}

resource "aws_lambda_permission" "apigw_lambda" {
  statement_id  = "AllowExecutionFromAPIGateway"
  action        = "lambda:InvokeFunction"
  function_name = var.lambda_function_name
  principal     = "apigateway.amazonaws.com"

  source_arn = "arn:aws:execute-api:${data.aws_region.current.name}:${data.aws_caller_identity.current.account_id}:${aws_api_gateway_rest_api.api.id}/*/ANY/*"
}

resource "aws_api_gateway_deployment" "default" {
  rest_api_id = aws_api_gateway_rest_api.api.id

  triggers = {
    # NOTE: The configuration below will satisfy ordering considerations,
    #       but not pick up all future REST API changes. More advanced patterns
    #       are possible, such as using the filesha1() function against the
    #       Terraform configuration file(s) or removing the .id references to
    #       calculate a hash against whole resources. Be aware that using whole
    #       resources will show a difference after the initial implementation.
    #       It will stabilize to only change when resources change afterwards.
    redeployment = sha1(jsonencode([
      aws_api_gateway_resource.default.id,
      aws_api_gateway_method.get_s3_default.id,
      [for val in aws_api_gateway_resource.lambda_proxy : val.id],
      [for val in aws_api_gateway_method.lambda_proxy : val.id],
      [for val in aws_api_gateway_method_response.lambda_proxy_response_200 : val.id],
      [for val in aws_api_gateway_integration.lambda_proxy : val.id],
    ]))
  }

  lifecycle {
    create_before_destroy = true
  }
}

resource "aws_api_gateway_stage" "default" {
  rest_api_id   = aws_api_gateway_rest_api.api.id
  deployment_id = aws_api_gateway_deployment.default.id
  stage_name    = "default"
}

resource "aws_api_gateway_base_path_mapping" "example" {
  api_id      = aws_api_gateway_rest_api.api.id
  stage_name  = aws_api_gateway_stage.default.stage_name
  domain_name = "api.${var.deployment_env}.${var.root_domain_name}" # TODO: Create Domain Name Resource
}