output "ui_hostname" {
  value       = aws_s3_bucket.public_ui.bucket
  description = "The hostname for the UI, e.g., 'koalabot.uk'."
}