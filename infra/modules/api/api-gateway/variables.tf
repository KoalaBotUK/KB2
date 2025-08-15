variable "deployment_env" {
  type = string
}

variable "lambda_function_invoke_arn" {
  type = string
}

variable "lambda_function_name" {
  type = string
}

variable "root_domain_name" {
  type    = string
}

variable "ui_hostname" {
  type = string
  description = "The hostname for the UI, e.g., 'koalabot.uk'. This is used to set up the API Gateway custom domain name."
}