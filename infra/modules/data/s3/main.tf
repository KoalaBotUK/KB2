resource "aws_s3_bucket" "public_ui" {
  bucket = "${var.deployment_env}.koalabot.uk"

  tags = {
    Name        = var.deployment_env == "prod" ? "koalabot.uk" : "${var.deployment_env}.koalabot.uk",
    Environment = var.deployment_env
  }
}

resource "aws_s3_bucket_policy" "bucket_policy" {
  bucket = aws_s3_bucket.public_ui.id
  policy = jsonencode({
    Version = "2012-10-17",
    Statement = [
      {
        "Sid": "PublicReadGetObject",
        "Effect": "Allow",
        "Principal": "*",
        "Action": "s3:GetObject",
        "Resource": "arn:aws:s3:::${aws_s3_bucket.public_ui.bucket}/*",
        "Condition": {
          "IpAddress": {
            "aws:SourceIp": [
              "173.245.48.0/20",
              "103.21.244.0/22",
              "103.22.200.0/22",
              "103.31.4.0/22",
              "141.101.64.0/18",
              "108.162.192.0/18",
              "190.93.240.0/20",
              "188.114.96.0/20",
              "197.234.240.0/22",
              "198.41.128.0/17",
              "162.158.0.0/15",
              "104.16.0.0/13",
              "104.24.0.0/14",
              "172.64.0.0/13",
              "131.0.72.0/22"
            ]
          }
        }
      }
    ]
  })
}

resource "aws_s3_bucket_website_configuration" "default" {
  bucket = aws_s3_bucket.public_ui.id

  index_document {
    suffix = "index.html"
  }

  error_document {
    key = "index.html"
  }
}

output "website_url" {
  value = aws_s3_bucket_website_configuration.default.website_endpoint
}