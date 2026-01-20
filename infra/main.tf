terraform {
  cloud {
    organization = "JayDwee"

    workspaces {
      project = "kb2"
      tags = ["source:github.com/jaydwee/kb2-infrastructure"]
    }
  }

  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 6.27"
    }
  }

  required_version = ">= 1.3.0"
}

provider "aws" {}

# module "s3" {
#   source = "./modules/data/s3"
#   deployment_env = var.deployment_env
# }

module "dsql" {
  source = "./modules/data/dsql"
  deployment_env = var.deployment_env
}

module "lambda" {
  source         = "./modules/compute/lambda"
  deployment_env = var.deployment_env
  discord_bot_token = var.discord_bot_token
  discord_public_key = var.discord_public_key
  dsql_user = "admin"
  dsql_endpoint = module.dsql.dsql_endpoint
  dsql_arn = module.dsql.dsql_arn
}

module "s3" {
  source = "./modules/data/s3"
  deployment_env = var.deployment_env
}

module "api" {
  source = "./modules/api/api-gateway"
  deployment_env = var.deployment_env
  root_domain_name = var.root_domain_name
  lambda_function_invoke_arn = module.lambda.lambda_function_invoke_arn
  lambda_function_name = module.lambda.lambda_function_name
  ui_hostname = module.s3.ui_hostname
}

module "dynamodb" {
  source = "./modules/data/dynamodb"
  deployment_env = var.deployment_env
}

module "ses" {
  source = "./modules/data/ses"
  deployment_env = var.deployment_env
}