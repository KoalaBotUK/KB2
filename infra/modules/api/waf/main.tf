
resource "aws_wafv2_web_acl" "default" {
  name  = "kb2-web-acl-${var.deployment_env}"
  scope = "REGIONAL"

  default_action {
    allow {}
  }

  visibility_config {
    cloudwatch_metrics_enabled = true
    metric_name                = "kb2-web-acl-metric-${var.deployment_env}"
    sampled_requests_enabled   = false
  }


  # Add the AWS Bot Control Managed Rule Group
  rule {
    name     = "AWS-AWSManagedRulesBotControlRuleSet"
    priority = 1

    override_action {
      none {}
    }

    statement {
      managed_rule_group_statement {
        name        = "AWSManagedRulesBotControlRuleSet"
        vendor_name = "AWS"

        managed_rule_group_configs {
          aws_managed_rules_bot_control_rule_set {
            inspection_level = "COMMON"
          }
        }
      }
    }

    visibility_config {
      cloudwatch_metrics_enabled = true
      metric_name                = "AWSManagedRulesBotControlRuleSet-metric"
      sampled_requests_enabled   = false
    }
  }
}

resource "aws_wafv2_web_acl_association" "default" {
  resource_arn = var.api_gw_arn
  web_acl_arn  = aws_wafv2_web_acl.default.arn
}