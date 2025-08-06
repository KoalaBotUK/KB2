terraform {
  cloud {
    organization = "JayDwee"

    workspaces {
      project = "auther"
      tags = ["source:github.com/jaydwee/auther-infrastructure"]
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

module "s3" {
  source = "./modules/data/s3"
  deployment_env = var.deployment_env
}