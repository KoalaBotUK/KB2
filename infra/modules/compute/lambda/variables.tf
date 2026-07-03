variable "deployment_env" {
  type = string
}

variable "discord_bot_token" {
  type = string
}

variable "discord_public_key" {
  type = string
}

variable "email_link_signing_key" {
  type = string
}

variable "ui_hostname" {
  type        = string
  description = "The hostname for the UI, e.g., 'koalabot.uk'. Used to restrict CORS on the API to the first-party UI origin."
}

variable "dsql_user" {
  type = string
}

variable "dsql_endpoint" {
  type = string
}

variable "dsql_arn" {
  type = string
}

variable "sqs_arn" {
  type = string
}

variable "sqs_url" {
  type = string
}
