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
      version = "~> 5.0"
    }
  }

  required_version = ">= 1.3.0"
}

provider "aws" {}

# module "s3" {
#   source = "./modules/data/s3"
#   deployment_env = var.deployment_env
# }

module "lambda" {
  count = var.deployment_env == "local" ? 0 : 1
  source         = "./modules/compute/lambda"
  deployment_env = var.deployment_env
  discord_bot_token = var.discord_bot_token
  discord_public_key = var.discord_public_key
}

module "s3" {
  count = var.deployment_env == "local" ? 0 : 1
  source = "./modules/data/s3"
  deployment_env = var.deployment_env
}

module "api" {
  count = var.deployment_env == "local" ? 0 : 1
  source = "./modules/api/api-gateway"
  deployment_env = var.deployment_env
  root_domain_name = var.root_domain_name
  lambda_function_invoke_arn = module.lambda[0].lambda_function_invoke_arn
  lambda_function_name = module.lambda[0].lambda_function_name
  ui_hostname = module.s3[0].ui_hostname
}

module "dynamodb" {
  source = "./modules/data/dynamodb"
  deployment_env = var.deployment_env
}

module "ses" {
  source = "./modules/data/ses"
  deployment_env = var.deployment_env
}