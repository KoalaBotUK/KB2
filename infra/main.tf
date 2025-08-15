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
  source         = "./modules/compute/lambda"
  deployment_env = var.deployment_env
}

module "api" {
  source = "./modules/api/api-gateway"
  deployment_env = var.deployment_env
  root_domain_name = var.root_domain_name
  lambda_function_invoke_arn = module.lambda.lambda_function_invoke_arn
  lambda_function_name = module.lambda.lambda_function_name
}

module "dynamodb" {
  source = "./modules/data/dynamodb"
  deployment_env = var.deployment_env
}