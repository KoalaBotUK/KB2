// Read from email-template.html
data "local_file" "verify_template_html" {
    filename = "${path.module}/resources/email-template.html"
}

data "local_file" "verify_template_txt" {
  filename = "${path.module}/resources/email-template.txt"
}

resource "aws_ses_template" "verify_template" {
  name    = "kb2-verify-templacte-${var.deployment_env}"
  subject = "Verify your email with Koala!"
  html    = data.local_file.verify_template_html.content
  text    = data.local_file.verify_template_txt.content
}